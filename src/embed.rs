use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct Thumbnail {
    pub(crate) url: String,
}

#[derive(Serialize)]
pub(crate) struct EmbedFooter {
    pub(crate) text: String,
}

#[derive(Serialize)]
pub(crate) struct EmbedField {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) inline: Option<bool>,
}

#[derive(Serialize)]
pub(crate) struct Embed {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) url: Option<String>,
    pub(crate) thumbnail: Option<Thumbnail>,
    pub(crate) footer: Option<EmbedFooter>,
    pub(crate) fields: Vec<EmbedField>,
    pub(crate) color: Option<u32>,
}
