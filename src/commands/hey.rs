use crate::command::Command;
use crate::error::InteractionError;
use crate::interaction::{
    ApplicationCommandInteractionDataOption, ApplicationCommandOption,
    ApplicationCommandOptionChoice, ApplicationCommandOptionType,
    InteractionApplicationCommandCallbackData, Member,
};

use async_trait::async_trait;

pub(crate) struct Hey {}

#[async_trait(?Send)]
impl Command for Hey {
    async fn respond(
        &self,
        _member: &Option<Member>,
        options: &Option<Vec<ApplicationCommandInteractionDataOption>>,
        _ctx: &mut worker::RouteContext<()>,
    ) -> Result<InteractionApplicationCommandCallbackData, InteractionError> {
        let name = options
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .next()
            .cloned()
            .unwrap_or(ApplicationCommandInteractionDataOption {
                name: "name".into(),
                ty: ApplicationCommandOptionType::String,
                focused: Some(false),
                value: Some("Someone".into()),
            })
            .value
            .unwrap_or("Someone".into());

        Ok(InteractionApplicationCommandCallbackData {
            content: Some(format!("Hey, {}!", name)),
            choices: None,
            embeds: None,
        })
    }

    fn name(&self) -> String {
        "hey".into()
    }

    fn description(&self) -> String {
        "Say Hey to the user!".into()
    }

    fn options(&self) -> Option<Vec<ApplicationCommandOption>> {
        Some(vec![ApplicationCommandOption {
            name: "name".into(),
            autocomplete: Some(true),
            description: "The user you want to say hey to".into(),
            required: Some(true),
            ty: ApplicationCommandOptionType::String,
            choices: None,
        }])
    }

    async fn autocomplete(
        &self,
        _options: &Option<Vec<ApplicationCommandInteractionDataOption>>,
        _ctx: &mut worker::RouteContext<()>,
    ) -> Result<InteractionApplicationCommandCallbackData, InteractionError> {
        Ok(InteractionApplicationCommandCallbackData {
            content: None,
            embeds: None,
            choices: Some(vec![
                ApplicationCommandOptionChoice {
                    name: "loki".into(),
                    value: "Loki".into(),
                },
                ApplicationCommandOptionChoice {
                    name: "icecream".into(),
                    value: "IceCream".into(),
                },
                ApplicationCommandOptionChoice {
                    name: "yoroshi".into(),
                    value: "Yoroshi".into(),
                },
            ]),
        })
    }
}
