use base64::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::node::Data;

#[derive(Clone, Deserialize, Serialize)]
pub enum Transformer {
    Split { pattern: String },
    Join { separator: String },
    Find { pattern: String },
    Replace { pattern: String, replacer: String },
    Slice { from: usize, to: usize },
    Encode { encoding: Encoding },
    Decode { encoding: Encoding },
    Uppercase,
    Lowercase,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Encoding {
    Base64,
    Base64UrlSafe,
    URL,
}

impl Transformer {
    pub fn transform(&self, data: &Data) -> Data {
        match data {
            Data::Text(text) => match self {
                Transformer::Split { pattern } => {
                    Data::List(text.split(pattern).map(|p| Data::Text(p.into())).collect())
                }
                Transformer::Find { pattern } => {
                    if let Ok(re) = Regex::new(pattern) {
                        Data::List(
                            re.find_iter(text)
                                .map(|m| Data::Text(m.as_str().into()))
                                .collect(),
                        )
                    } else {
                        Data::Error("Invalid pattern".into())
                    }
                }
                Transformer::Replace { pattern, replacer } => {
                    if let Ok(re) = Regex::new(pattern) {
                        Data::Text(re.replace_all(text, replacer).into())
                    } else {
                        Data::Error("Invalid pattern".into())
                    }
                }
                Transformer::Slice { from, to } => {
                    Data::Text(text[(*from).min(text.len())..(*to).min(text.len())].into())
                }
                Transformer::Join { .. } => Data::Text(text.into()),
                Transformer::Encode { encoding } => match encoding {
                    Encoding::Base64 => Data::Text(BASE64_STANDARD.encode(text)),
                    Encoding::Base64UrlSafe => Data::Text(BASE64_URL_SAFE.encode(text)),
                    Encoding::URL => Data::Text(urlencoding::encode(text).into()),
                },
                Transformer::Decode { encoding } => match encoding {
                    Encoding::Base64 => match BASE64_STANDARD.decode(text) {
                        Ok(text) => Data::Text(String::from_utf8_lossy(&text).into()),
                        Err(err) => Data::Error(err.to_string()),
                    },
                    Encoding::Base64UrlSafe => match BASE64_URL_SAFE.decode(text) {
                        Ok(text) => Data::Text(String::from_utf8_lossy(&text).into()),
                        Err(err) => Data::Error(err.to_string()),
                    },
                    Encoding::URL => match urlencoding::decode(text) {
                        Ok(text) => Data::Text(text.into()),
                        Err(err) => Data::Error(err.to_string()),
                    },
                },
                Transformer::Uppercase => Data::Text(text.to_uppercase()),
                Transformer::Lowercase => Data::Text(text.to_lowercase()),
            },
            Data::List(data_vec) => match self {
                Transformer::Join { separator } => {
                    let mut texts = Vec::new();
                    fn collect(d: &Data, sep: &str, out: &mut Vec<String>) -> Option<String> {
                        match d {
                            Data::Text(t) => {
                                out.push(t.into());
                                None
                            }
                            Data::Error(_) => Some("Input error".into()),
                            Data::List(list) => {
                                for item in list {
                                    if let Some(err) = collect(item, sep, out) {
                                        return Some(err);
                                    }
                                }
                                None
                            }
                        }
                    }
                    if let Some(err) = collect(data, separator, &mut texts) {
                        Data::Error(err)
                    } else {
                        Data::Text(texts.join(separator))
                    }
                }
                _ => Data::List(data_vec.iter().map(|d| self.transform(&d)).collect()),
            },
            Data::Error(_) => Data::Error("Input error".into()),
        }
    }
}
