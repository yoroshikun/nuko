use std::collections::HashMap;

use chrono;
use rasciigraph::{plot, Config};
use reqwest;
use serde::Deserialize;
use serde_json::Value;
use worker::kv::KvStore;
use worker::RouteContext;

use crate::embed::{Embed, EmbedField};

pub struct XEOptions {
    set_default: Option<bool>,
    from: Option<String>,
    to: Option<String>,
    amount: Option<String>,
}

#[derive(Clone)]
pub struct Request {
    from: String,
    to: String,
    amount: f64,
    precision: usize,
    dates: TimeseriesRequest,
}

#[derive(Deserialize, Debug)]
pub struct FixerResponse {
    base: String,
    date: String,
    rates: Value,
    success: bool,
    timestamp: u64,
}

#[derive(Deserialize, Debug)]
pub struct FixerTimeseriesResponse {
    base: String,
    start_date: String,
    end_date: String,
    rates: TimeseriesResponse,
    success: bool,
    timeseries: bool,
}

pub type TimeseriesResponse = HashMap<String, HashMap<String, f64>>;

#[derive(Clone)]
pub struct TimeseriesRequest {
    start_date: String,
    end_date: String,
}

pub struct XEClient {
    client: reqwest::Client,
    request: Request,
    rate: Option<f64>,
    timeseries: Option<TimeseriesResponse>,
}

impl XEClient {
    pub async fn new(
        from: Option<&String>,
        to: Option<&String>,
        amount: Option<&String>,
        precision: Option<&String>,
        dates: Option<&String>,
        kv: &KvStore,
        username: &String,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            request: Request {
                from: XEClient::resolve_from(from, kv, username)
                    .await
                    .unwrap_or("USD".into()),
                to: XEClient::resolve_to(to, kv, username)
                    .await
                    .unwrap_or("JPY".into()),
                amount: XEClient::resolve_amount(amount).await,
                precision: XEClient::resolve_precision(precision, kv, username)
                    .await
                    .unwrap_or(4),
                dates: XEClient::resolve_dates(dates),
            },
            rate: None,
            timeseries: None,
        }
    }

    async fn resolve_from(
        from: Option<&String>,
        kv: &KvStore,
        username: &String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if from.is_some() {
            return Ok(from.unwrap_or(&"USD".into()).to_owned());
        }

        let key = format!("{}:currency_from", username);
        Ok(kv
            .get(key.as_str())
            .text()
            .await?
            .unwrap_or("USD".into())
            .into())
    }

    async fn resolve_to(
        to: Option<&String>,
        kv: &KvStore,
        username: &String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if to.is_some() {
            return Ok(to.unwrap_or(&"JPY".into()).to_owned());
        }

        let key = format!("{}:currency_to", username);
        Ok(kv
            .get(key.as_str())
            .text()
            .await?
            .unwrap_or("JPY".into())
            .into())
    }

    async fn resolve_precision(
        precision: Option<&String>,
        kv: &KvStore,
        username: &String,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        if precision.is_some() {
            return Ok(precision
                .unwrap_or(&"4".into())
                .parse::<usize>()
                .unwrap_or(4));
        }

        let key = format!("{}:currency_precision", username);
        Ok(kv
            .get(key.as_str())
            .text()
            .await?
            .unwrap_or("4".into())
            .parse::<usize>()
            .unwrap_or(4))
    }

    async fn resolve_amount(amount: Option<&String>) -> f64 {
        amount.unwrap_or(&"1".into()).parse::<f64>().unwrap_or(1.)
    }

    fn resolve_dates(dates: Option<&String>) -> TimeseriesRequest {
        let default_end_date = chrono::Utc::today()
            .naive_utc()
            .format("%Y-%m-%d")
            .to_string();
        let default_start_date = (chrono::Utc::today() - chrono::Duration::days(14))
            .naive_utc()
            .format("%Y-%m-%d")
            .to_string();

        // Split the request by - if exists
        let timeseries = match dates {
            Some(dates) => {
                let split_dates = dates.split('_').collect::<Vec<&str>>();

                TimeseriesRequest {
                    start_date: match split_dates.get(0) {
                        Some(start_date) => start_date.to_string(),
                        None => default_start_date,
                    },
                    end_date: match split_dates.get(1) {
                        Some(end_date) => end_date.to_string(),
                        None => default_end_date,
                    },
                }
            }
            None => TimeseriesRequest {
                start_date: default_start_date,
                end_date: default_end_date,
            },
        };

        timeseries
    }

    pub async fn get_timeseries(
        &mut self,
        ctx: &mut RouteContext<()>,
        kv: &KvStore,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let timeseries_cache_key = format!(
            "{}_{}_{}_{}",
            self.request.dates.start_date,
            self.request.dates.end_date,
            self.request.from,
            self.request.to
        );

        // Get the cache
        let timeseries_cache = kv
            .get(format!("timeseries_cache:{}", timeseries_cache_key).as_str())
            .text()
            .await?;

        // Get cache if exists
        if timeseries_cache.is_some() {
            let timeseries_cache = timeseries_cache.unwrap();
            let timeseries_cache: TimeseriesResponse =
                serde_json::from_str(timeseries_cache.as_str())?;

            self.timeseries = Some(timeseries_cache);
            return Ok(());
        };

        let api_key = ctx.var("CURR_CONV_TOKEN")?.to_string();
        let res = self
            .client
            .get(format!(
                "https://api.apilayer.com/fixer/timeseries?symbols={}&base={}&start_date={}&end_date={}",
                self.request.to, self.request.from, self.request.dates.start_date, self.request.dates.end_date
            ))
            .header("apiKey", api_key)
            .send()
            .await?
            .json::<FixerTimeseriesResponse>()
            .await?;

        worker::console_log!("Currency converter timeseries body : {:?}", res);

        let rates = res.rates;

        // Set the cache
        if !rates.is_empty() {
            kv.put(
                format!("timeseries_cache:{}", timeseries_cache_key.clone()).as_str(),
                serde_json::to_string(&rates)
                    .expect("Could not convert timeseries rates to json string")
                    .as_str(), // Serialise this
            )?
            .execute()
            .await?;
            self.timeseries = Some(rates);
        }

        Ok(())
    }

    pub async fn get_rate(
        &mut self,
        ctx: &mut RouteContext<()>,
        kv: &KvStore,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conversion_key = format!("{}_{}", self.request.from, self.request.to);

        // Get rate cache and compare its timestamp
        // If over 4 hours ago fetch agains
        let rate_cache = kv
            .get(format!("cache:{}", conversion_key).as_str())
            .text()
            .await?;
        if rate_cache.is_some() {
            let rate_cache = rate_cache.unwrap();
            let rate_cache: Value = serde_json::from_str(rate_cache.as_str())?;
            let timestamp = rate_cache["timestamp"].as_u64().unwrap();
            let now = chrono::Utc::now().timestamp() as u64;
            if now - timestamp < 14400 {
                self.rate = Some(rate_cache["rate"].as_f64().unwrap());
                return Ok(());
            }
        }

        let api_key = ctx.var("CURR_CONV_TOKEN")?.to_string();
        let res = self
            .client
            .get(format!(
                "https://api.apilayer.com/fixer/latest?symbols={}&base={}",
                self.request.to, self.request.from
            ))
            .header("apiKey", api_key)
            .send()
            .await?
            .json::<FixerResponse>()
            .await?;

        worker::console_log!("Currency converter body : {:?}", res);

        let rate: Option<f64> = res.rates[self.request.to.clone().as_str()].as_f64();

        // Cache rate
        if rate.is_some() {
            let rate = rate.unwrap();
            let rate_cache = serde_json::json!({
                "rate": rate,
                "timestamp": chrono::Utc::now().timestamp()
            });
            kv.put(
                format!("cache:{}", conversion_key.clone()).as_str(),
                rate_cache.to_string().as_str(),
            )?
            .execute()
            .await?;
            self.rate = Some(rate);
        }

        self.rate = rate;

        Ok(())
    }

    pub(crate) async fn set_default(
        &self,
        kv: &KvStore,
        username: &String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let to_key = format!("{}:currency_to", username);
        let from_key = format!("{}:currency_from", username);
        let precision_key = format!("{}:currency_precision", username);

        kv.put(to_key.as_str(), self.request.to.clone())?
            .execute()
            .await?;
        kv.put(from_key.as_str(), self.request.from.clone())?
            .execute()
            .await?;
        kv.put(precision_key.as_str(), self.request.precision)?
            .execute()
            .await?;

        Ok(())
    }

    fn get_xe(&self) -> String {
        format!(
            "{xe:.precision$}",
            xe = (self.rate.unwrap_or(1.) * self.request.amount),
            precision = (if self.request.precision <= 12 {
                self.request.precision
            } else {
                12
            })
        )
    }

    pub(crate) fn construct_rate_embed(&self) -> Embed {
        Embed {
            title: "Exchange Rate".into(),
            description: format!(
                "{} {} --> {} {}",
                self.request.amount,
                self.request.from,
                self.get_xe(),
                self.request.to
            ),
            url: None,
            thumbnail: None,
            footer: None,
            fields: vec![],
            color: Some(0xfdc835),
        }
    }

    pub(crate) fn construct_timeseries_embed(&self) -> Embed {
        // Turn the timeseries into a vec of values
        let mut timeseries_vec: Vec<f64> = vec![];

        for (_, v) in self.timeseries.clone().unwrap().iter() {
            timeseries_vec.push(v[self.request.to.clone().as_str()]);
        }

        // Find max in vec
        let max = timeseries_vec.iter().cloned().fold(f64::MIN, f64::max);
        // Find min in vec
        let min = timeseries_vec.iter().cloned().fold(f64::MAX, f64::min);

        Embed {
            title: "Exchange Rate Timeseries".into(),
            description: format!(
                "{}",
                plot(
                    timeseries_vec,
                    Config::default().with_offset(10).with_height(10)
                )
            )
            .to_string(),
            url: None,
            thumbnail: None,
            footer: None,
            fields: vec![
                EmbedField {
                    name: "Min".to_string(),
                    inline: Some(true),
                    value: format!("{:.2}", min),
                },
                EmbedField {
                    name: "Max".to_string(),
                    inline: Some(true),
                    value: format!("{:.2}", max),
                },
            ],
            color: Some(0xfdc835),
        }
    }
}
