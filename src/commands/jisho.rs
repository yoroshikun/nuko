use crate::command::Command;
use crate::error::InteractionError;
use crate::helpers::jisho_client::JishoClient;
use crate::interaction::{
    ApplicationCommandInteractionDataOption, ApplicationCommandOption,
    ApplicationCommandOptionChoice, ApplicationCommandOptionType,
    InteractionApplicationCommandCallbackData, Member,
};

use async_trait::async_trait;

pub(crate) struct Jisho {}

#[async_trait(?Send)]
impl Command for Jisho {
    async fn respond(
        &self,
        _member: &Option<Member>,
        options: &Option<Vec<ApplicationCommandInteractionDataOption>>,
        _ctx: &mut worker::RouteContext<()>,
    ) -> Result<InteractionApplicationCommandCallbackData, InteractionError> {
        let word = options
            .as_ref()
            .expect("Options not provided")
            .into_iter()
            .next()
            .expect("Word not provided")
            .clone()
            .value
            .expect("Word not provided");

        let mut jisho_client = JishoClient::new(word);
        jisho_client
            .api_get_word()
            .await
            .expect("Unable to get word from jisho api");
        let embed = jisho_client.construct_embed().await;

        Ok(InteractionApplicationCommandCallbackData {
            content: None,
            choices: None,
            embeds: Some(vec![embed]),
        })
    }

    fn name(&self) -> String {
        "jisho".into()
    }

    fn description(&self) -> String {
        "Jisho a word!".into()
    }

    fn options(&self) -> Option<Vec<ApplicationCommandOption>> {
        Some(vec![ApplicationCommandOption {
            name: "word".into(),
            autocomplete: Some(false),
            description: "The word you want to jisho".into(),
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
            choices: None,
        })
    }
}
