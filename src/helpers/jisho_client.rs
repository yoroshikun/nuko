use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::embed::{Embed, EmbedField, EmbedFooter};

#[derive(Deserialize)]
pub struct Response {
    meta: Meta,
    data: Vec<Data>,
}

#[derive(Deserialize, Clone)]
pub struct Data {
    slug: String,
    is_common: bool,
    tags: Vec<String>,
    jlpt: Vec<String>,
    japanese: Vec<Japanese>,
    senses: Vec<Sense>,
    attribution: Attribution,
}

#[derive(Deserialize, Clone)]
pub struct Attribution {
    jmdict: bool,
    jmnedict: bool,
    dbpedia: Value,
}

#[derive(Deserialize, Clone)]
pub struct Japanese {
    word: Option<String>,
    reading: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct Sense {
    english_definitions: Vec<String>,
    parts_of_speech: Vec<String>,
    links: Vec<Link>,
    tags: Vec<String>,
    restrictions: Vec<String>,
    see_also: Vec<String>,
    antonyms: Vec<Value>,
    source: Vec<Source>,
    info: Vec<String>,
    sentences: Option<Vec<Value>>,
}

#[derive(Deserialize, Clone)]
pub struct Link {
    text: String,
    url: String,
}

#[derive(Deserialize, Clone)]
pub struct Source {
    language: String,
    word: String,
}

#[derive(Deserialize)]
pub struct Meta {
    status: usize,
}

#[derive(Serialize)]
pub struct RequestOptions {
    word: String,
}

pub struct JishoClient {
    client: reqwest::Client,
    options: RequestOptions,
    response: Option<Response>,
}

impl JishoClient {
    pub fn new(word: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            options: RequestOptions { word },
            response: None,
        }
    }

    pub async fn api_get_word(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let res = self
            .client
            .get(format!(
                "https://jisho.org/api/v1/search/words?keyword={}",
                self.options.word
            ))
            .send()
            .await?
            .json::<Response>()
            .await?;

        self.response = Some(res);

        Ok(())
    }

    pub(crate) async fn construct_embed(&self) -> Embed {
        match &self.response {
            Some(res) => match res.data.clone().into_iter().next() {
                Some(data) => {
                    let english = match data.senses.clone().into_iter().next() {
                        Some(english) => english
                            .english_definitions
                            .into_iter()
                            .next()
                            .unwrap_or("Unknown English Translation".into()),
                        None => "Unknown English Translation".into(),
                    };

                    let japanese = match data.japanese.clone().into_iter().next() {
                        Some(japanese) => japanese.word.unwrap_or("No Kana".into()),
                        None => "No Kana".into(),
                    };

                    let reading = match data.japanese.clone().into_iter().next() {
                        Some(japanese) => japanese.reading.unwrap_or("No Reading".into()),
                        None => "No Reading".into(),
                    };

                    let extras = format!(
                        "[📗](https://jisho.org/word/{}) | [🔍](https://jisho.org/search/{})",
                        data.slug, self.options.word
                    );

                    Embed {
                        title: "".into(),
                        description: "".into(),
                        url: None,
                        thumbnail: None,
                        footer: None,
                        fields: vec![
                            EmbedField {
                                name: "Word searched".into(),
                                value: self.options.word.to_owned(),
                                inline: Some(false),
                            },
                            EmbedField {
                                name: "English".into(),
                                value: english,
                                inline: Some(true),
                            },
                            EmbedField {
                                name: "Japanese".into(),
                                value: japanese.to_owned(),
                                inline: Some(true),
                            },
                            EmbedField {
                                name: "Reading".into(),
                                value: reading,
                                inline: Some(true),
                            },
                            EmbedField {
                                name: "Extras".into(),
                                value: extras,
                                inline: Some(false),
                            },
                        ],
                        color: Some(0x00FF00),
                    }
                }
                None => Embed {
                    title: "Error".into(),
                    description: "No data found for that word".into(),
                    url: None,
                    thumbnail: None,
                    footer: None,
                    fields: vec![],
                    color: Some(0xFF0000),
                },
            },
            None => Embed {
                title: "Error".into(),
                description: "No data found for that word".into(),
                url: None,
                thumbnail: None,
                footer: None,
                fields: vec![],
                color: Some(0xFF0000),
            },
        }
    }
}
