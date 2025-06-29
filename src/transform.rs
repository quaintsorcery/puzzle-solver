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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split() {
        let transformer = Transformer::Split {
            pattern: " ".into(),
        };

        test_transformer(
            &transformer,
            Data::Text("Sample Text".into()),
            Data::List(vec![Data::Text("Sample".into()), Data::Text("Text".into())]),
        );

        test_transformer(
            &transformer,
            Data::List(vec![
                Data::Text("Sample Text".into()),
                Data::Text("Another Sample Text".into()),
            ]),
            Data::List(vec![
                Data::List(vec![Data::Text("Sample".into()), Data::Text("Text".into())]),
                Data::List(vec![
                    Data::Text("Another".into()),
                    Data::Text("Sample".into()),
                    Data::Text("Text".into()),
                ]),
            ]),
        );
    }

    #[test]
    fn test_join() {
        let transformer = Transformer::Join {
            separator: " ".into(),
        };

        test_transformer(
            &transformer,
            Data::List(vec![Data::Text("Sample".into()), Data::Text("Text".into())]),
            Data::Text("Sample Text".into()),
        );

        test_transformer(
            &transformer,
            Data::List(vec![
                Data::List(vec![Data::Text("Sample".into()), Data::Text("Text".into())]),
                Data::List(vec![
                    Data::Text("Another".into()),
                    Data::Text("Sample".into()),
                    Data::Text("Text".into()),
                ]),
            ]),
            Data::Text("Sample Text Another Sample Text".into()),
        );
    }

    #[test]
    fn test_find() {
        let transformer = Transformer::Find {
            pattern: "Text".into(),
        };

        test_transformer(
            &transformer,
            Data::Text("Sample Text".into()),
            Data::List(vec![Data::Text("Text".into())]),
        );

        test_transformer(
            &transformer,
            Data::List(vec![
                Data::Text("Sample Text".into()),
                Data::Text("Another Sample Text".into()),
            ]),
            Data::List(vec![
                Data::List(vec![Data::Text("Text".into())]),
                Data::List(vec![Data::Text("Text".into())]),
            ]),
        );
    }

    #[test]
    fn test_replace() {
        let transformer = Transformer::Replace {
            pattern: "Sample".into(),
            replacer: "Test".into(),
        };

        test_transformer(
            &transformer,
            Data::Text("Sample Text".into()),
            Data::Text("Test Text".into()),
        );

        test_transformer(
            &transformer,
            Data::List(vec![
                Data::Text("Sample Text".into()),
                Data::Text("Another Sample Text".into()),
            ]),
            Data::List(vec![
                Data::Text("Test Text".into()),
                Data::Text("Another Test Text".into()),
            ]),
        );
    }

    #[test]
    fn test_slice() {
        let transformer = Transformer::Slice { from: 1, to: 6 };

        test_transformer(
            &transformer,
            Data::Text("Sample Text".into()),
            Data::Text("ample".into()),
        );

        test_transformer(
            &transformer,
            Data::List(vec![
                Data::Text("Sample Text".into()),
                Data::Text("Another Sample Text".into()),
            ]),
            Data::List(vec![Data::Text("ample".into()), Data::Text("nothe".into())]),
        );
    }

    // TODO: encode and decode tests

    #[test]
    fn test_uppercase() {
        let transformer = Transformer::Uppercase;

        test_transformer(
            &transformer,
            Data::Text("Sample Text".into()),
            Data::Text("SAMPLE TEXT".into()),
        );

        test_transformer(
            &transformer,
            Data::List(vec![Data::Text("Sample".into()), Data::Text("Text".into())]),
            Data::List(vec![Data::Text("SAMPLE".into()), Data::Text("TEXT".into())]),
        );
    }

    #[test]
    fn test_lowercase() {
        let transformer = Transformer::Lowercase;

        test_transformer(
            &transformer,
            Data::Text("Sample Text".into()),
            Data::Text("sample text".into()),
        );

        test_transformer(
            &transformer,
            Data::List(vec![Data::Text("Sample".into()), Data::Text("Text".into())]),
            Data::List(vec![Data::Text("sample".into()), Data::Text("text".into())]),
        );
    }

    fn test_transformer(transformer: &Transformer, input: Data, expected_output: Data) {
        assert_eq!(transformer.transform(&input), expected_output);
    }
}
