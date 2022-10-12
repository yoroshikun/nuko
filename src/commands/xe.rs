use crate::command::Command;
use crate::embed::Embed;
use crate::error::InteractionError;
use crate::helpers::xe_client::XEClient;
use crate::interaction::{
    ApplicationCommandInteractionDataOption, ApplicationCommandOption,
    ApplicationCommandOptionChoice, ApplicationCommandOptionType,
    InteractionApplicationCommandCallbackData, Member,
};

use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[derive(Debug, Display, EnumIter)]
enum CurrencyCodes {
    USD,
    EUR,
    JPY,
    BGN,
    BTC,
    CZK,
    DKK,
    GBP,
    SEK,
    CHF,
    AUD,
    BRL,
    CAD,
    CNY,
    HKD,
    INR,
    KRW,
    MXN,
    MYR,
    NZD,
    PHP,
    SGD,
}

use async_trait::async_trait;

pub(crate) struct XE {}

#[async_trait(?Send)]
impl Command for XE {
    async fn respond(
        &self,
        member: &Option<Member>,
        options: &Option<Vec<ApplicationCommandInteractionDataOption>>,
        ctx: &mut worker::RouteContext<()>,
    ) -> Result<InteractionApplicationCommandCallbackData, InteractionError> {
        let temp_vec = Vec::new();
        // Create a hash map of the options, so we can easily access them by name
        let options_hash_map = options
            .as_ref()
            .unwrap_or(&temp_vec)
            .iter()
            .map(|option| {
                (
                    option.name.as_str(),
                    option.value.clone().unwrap_or("".into()),
                )
            })
            .collect::<std::collections::HashMap<&str, String>>();

        let username = member.as_ref().unwrap().user.username.clone();

        let is_setting_defaults = match options_hash_map.get("set_defaults") {
            Some(default) => default.contains("True"),
            _ => false,
        };

        let is_getting_timeseries = match options_hash_map.get("timeseries") {
            Some(x) => true,
            _ => false,
        };

        let mut xe_client = XEClient::new(
            options_hash_map.get("from"),
            options_hash_map.get("to"),
            options_hash_map.get("amount"),
            options_hash_map.get("precision"),
            options_hash_map.get("timeseries"),
            &ctx.kv("exchange_defaults")?,
            &username,
        )
        .await;

        if is_setting_defaults {
            xe_client
                .set_default(&ctx.kv("exchange_defaults")?, &username)
                .await
                .expect("Unable to set defaults");

            return Ok(InteractionApplicationCommandCallbackData {
                content: None,
                choices: None,
                embeds: Some(vec![Embed {
                    title: "Exchange Rate".into(),
                    description: "Defaults have been updated".into(),
                    fields: vec![],
                    thumbnail: None,
                    color: Some(0xfdc835),
                    url: None,
                    footer: None,
                }]),
            });
        }

        if is_getting_timeseries {
            xe_client
                .get_timeseries(ctx, &ctx.kv("exchange_defaults")?)
                .await
                .expect("Unable to get timeseries");
            let embed = xe_client.construct_timeseries_embed();

            return Ok(InteractionApplicationCommandCallbackData {
                content: None,
                choices: None,
                embeds: Some(vec![embed]),
            });
        }

        xe_client
            .get_rate(ctx, &ctx.kv("exchange_defaults")?)
            .await
            .expect("Unable to get exchange rate from api");
        let embed = xe_client.construct_rate_embed();

        Ok(InteractionApplicationCommandCallbackData {
            content: None,
            choices: None,
            embeds: Some(vec![embed]),
        })
    }

    fn name(&self) -> String {
        "xe".into()
    }

    fn description(&self) -> String {
        "Convert from one currency to another".into()
    }

    fn options(&self) -> Option<Vec<ApplicationCommandOption>> {
        let choices = CurrencyCodes::iter()
            .map(|code| ApplicationCommandOptionChoice {
                name: code.to_string(),
                value: code.to_string(),
            })
            .collect::<Vec<ApplicationCommandOptionChoice>>();

        Some(vec![
            ApplicationCommandOption {
                name: "from".into(),
                autocomplete: Some(false),
                description: "The currency to convert from (Default AUD)".into(),
                required: Some(false),
                ty: ApplicationCommandOptionType::String,
                choices: Some(choices.clone()),
            },
            ApplicationCommandOption {
                name: "to".into(),
                autocomplete: Some(false),
                description: "The currency to convert to (Default JPY)".into(),
                required: Some(false),
                ty: ApplicationCommandOptionType::String,
                choices: Some(choices),
            },
            ApplicationCommandOption {
                name: "amount".into(),
                autocomplete: Some(false),
                description: "The amount of the currency (Number)".into(),
                required: Some(false),
                ty: ApplicationCommandOptionType::String,
                choices: None,
            },
            ApplicationCommandOption {
                name: "precision".into(),
                autocomplete: Some(false),
                description: "Precision of the decimal points (u8, max 12, default: 4)".into(),
                required: Some(false),
                ty: ApplicationCommandOptionType::String,
                choices: None,
            },
            ApplicationCommandOption {
                name: "timeseries".into(),
                autocomplete: Some(false),
                description:
                    "Get a timeseries graph of historical data (format: YYYY-MM-DD_YYYY_MM_DD)"
                        .into(),
                required: Some(false),
                ty: ApplicationCommandOptionType::String,
                choices: None,
            },
            ApplicationCommandOption {
                name: "set_defaults".into(),
                autocomplete: Some(false),
                description: "Set the default currencies for this user".into(),
                required: Some(false),
                ty: ApplicationCommandOptionType::String,
                choices: Some(vec![
                    ApplicationCommandOptionChoice {
                        name: "True".to_string(),
                        value: "True".to_string(),
                    },
                    ApplicationCommandOptionChoice {
                        name: "False".to_string(),
                        value: "False".to_string(),
                    },
                ]),
            },
        ])
    }

    async fn autocomplete(
        &self,
        _options: &Option<Vec<ApplicationCommandInteractionDataOption>>,
        _ctx: &mut worker::RouteContext<()>,
    ) -> Result<InteractionApplicationCommandCallbackData, InteractionError> {
        Ok(InteractionApplicationCommandCallbackData {
            content: None,
            embeds: None,
            choices: None,
        })
    }
}
