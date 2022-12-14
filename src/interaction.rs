use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::command::init_commands;
use crate::embed::Embed;
use crate::error::{Error, InteractionError};

#[derive(Deserialize_repr, Serialize)]
#[repr(u8)]
enum InteractionType {
    Ping = 1,
    ApplicationCommand = 2,
    MessageComponent = 3,
    ApplicationCommandAutoComplete = 4,
    ModalSubmit = 5,
}

#[allow(dead_code)]
#[derive(Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub(crate) enum InteractionResponseType {
    Pong = 1,
    // Acknowledge = 2,
    // ChannelMessage = 3,
    ChannelMessageWithSource = 4,
    ACKWithSource = 5,
    AutoCompleteResult = 8,
}

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct ApplicationCommandInteractionDataOption {
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) ty: ApplicationCommandOptionType,
    pub(crate) value: Option<String>,
    pub(crate) focused: Option<bool>,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct ApplicationCommandInteractionData {
    pub(crate) name: String,
    pub(crate) options: Option<Vec<ApplicationCommandInteractionDataOption>>,
}

#[derive(Serialize)]
pub(crate) struct InteractionApplicationCommandCallbackData {
    // https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-response-object-interaction-callback-data-structure
    pub(crate) content: Option<String>,
    pub(crate) choices: Option<Vec<ApplicationCommandOptionChoice>>,
    pub(crate) embeds: Option<Vec<Embed>>,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct Interaction {
    id: Option<String>,
    #[serde(rename = "type")]
    ty: InteractionType,
    data: Option<ApplicationCommandInteractionData>,
    token: String,
    guild_id: Option<String>,
    channel_id: Option<String>,
    application_id: Option<String>,
    member: Option<Member>,
    user: Option<User>,
    version: Option<u8>,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct Member {
    pub user: User,
    pub roles: Vec<String>,
    pub premium_since: Option<String>,
    pub permissions: String,
    pub pending: bool,
    pub nick: Option<String>,
    pub mute: bool,
    pub joined_at: String,
    pub is_pending: bool,
    pub deaf: bool,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct User {
    pub avatar: String,
    pub avatar_decoration: Option<String>,
    pub discriminator: String,
    pub id: String,
    pub public_flags: u32,
    pub username: String,
}

#[derive(Serialize_repr, Deserialize_repr, Clone)]
#[repr(u8)]
pub(crate) enum ApplicationCommandOptionType {
    // https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-type
    SubCommand = 1,
    SubCommandGroup = 2,
    String = 3,
    Boolean = 5,
}
#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct ApplicationCommandOption {
    // https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-structure
    pub(crate) name: String,
    pub(crate) description: String,
    #[serde(rename = "type")]
    pub(crate) ty: ApplicationCommandOptionType,
    pub(crate) choices: Option<Vec<ApplicationCommandOptionChoice>>,
    pub(crate) autocomplete: Option<bool>,
    pub(crate) required: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) struct ApplicationCommandOptionChoice {
    // https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-choice-structure
    pub(crate) name: String,
    pub(crate) value: String,
}

impl Interaction {
    fn data(&self) -> Result<&ApplicationCommandInteractionData, Error> {
        Ok(self
            .data
            .as_ref()
            .ok_or_else(|| Error::InvalidPayload("data not found".to_string()))?)
    }
}

#[derive(Serialize)]
pub struct InteractionResponse {
    #[serde(rename = "type")]
    pub(crate) ty: InteractionResponseType,
    pub(crate) data: Option<InteractionApplicationCommandCallbackData>,
}

impl Interaction {
    pub(crate) fn handle_ping(&self) -> InteractionResponse {
        return InteractionResponse {
            ty: InteractionResponseType::Pong,
            data: None,
        };
    }

    pub(crate) async fn handle_command(
        &self,
        ctx: &mut worker::RouteContext<()>,
    ) -> Result<InteractionResponse, InteractionError> {
        let data = self.data().map_err(|_| InteractionError::GenericError())?;
        let commands = init_commands();

        for boxed in commands.iter() {
            let com = &*boxed;
            if com.name() == data.name {
                let response = com.respond(&self.member, &data.options, ctx).await?;

                return Ok(InteractionResponse {
                    ty: InteractionResponseType::ChannelMessageWithSource,
                    data: Some(response),
                });
            }
        }
        Err(InteractionError::UnknownCommand(data.name.clone()))
    }

    pub(crate) async fn handle_autocomplete(
        &self,
        ctx: &mut worker::RouteContext<()>,
    ) -> Result<InteractionResponse, InteractionError> {
        let data = self.data().map_err(|_| InteractionError::GenericError())?;
        let commands = init_commands();

        for boxed in commands.iter() {
            let com = &*boxed;
            if com.name() == data.name {
                let response = com.autocomplete(&data.options, ctx).await?;

                return Ok(InteractionResponse {
                    ty: InteractionResponseType::AutoCompleteResult,
                    data: Some(response),
                });
            }
        }
        Err(InteractionError::UnknownCommand(data.name.clone()))
    }

    pub(crate) async fn perform(
        &self,
        ctx: &mut worker::RouteContext<()>,
    ) -> Result<InteractionResponse, Error> {
        match self.ty {
            InteractionType::Ping => Ok(self.handle_ping()),
            InteractionType::ApplicationCommand => self
                .handle_command(ctx)
                .await
                .map_err(Error::InteractionFailed),
            InteractionType::ApplicationCommandAutoComplete => self
                .handle_autocomplete(ctx)
                .await
                .map_err(Error::InteractionFailed),
            _ => Err(Error::InvalidPayload("Not implemented".into())),
        }
    }
}
