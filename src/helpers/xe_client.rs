use chrono;
use reqwest;
use serde::Deserialize;
use serde_json::Value;
use worker::kv::KvStore;
use worker::RouteContext;

use crate::embed::Embed;

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
}

#[derive(Deserialize, Debug)]
pub struct FixerResponse {
    base: String,
    date: String,
    rates: Value,
    success: bool,
    timestamp: u64,
}

pub struct XEClient {
    client: reqwest::Client,
    request: Request,
    rate: Option<f64>,
}

impl XEClient {
    pub async fn new(
        from: Option<&String>,
        to: Option<&String>,
        amount: Option<&String>,
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
            },
            rate: None,
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

    async fn resolve_amount(amount: Option<&String>) -> f64 {
        amount.unwrap_or(&"1".into()).parse::<f64>().unwrap_or(1.)
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

        kv.put(to_key.as_str(), self.request.to.clone())?
            .execute()
            .await?;
        kv.put(from_key.as_str(), self.request.from.clone())?
            .execute()
            .await?;

        Ok(())
    }

    fn get_xe(&self) -> String {
        format!("{:4}", (self.rate.unwrap_or(1.) * self.request.amount))
    }

    pub(crate) fn construct_embed(&self) -> Embed {
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
}
