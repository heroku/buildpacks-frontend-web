use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;
use std::fmt::Formatter;
use std::path::PathBuf;

pub(crate) const DEFAULT_DOC_ROOT: &str = "public";
pub(crate) const DEFAULT_DOC_INDEX: &str = "index.html";

#[derive(Deserialize, Default)]
pub(crate) struct HerokuWebServerConfig {
    pub(crate) root: Option<PathBuf>,
    pub(crate) index: Option<String>,
    pub(crate) errors: Option<ErrorsConfig>,
    #[serde(default, deserialize_with = "deserialize_headers")]
    pub(crate) headers: Option<Vec<Header>>,
}

#[derive(Deserialize, Eq, PartialEq, Debug, Default, Clone)]
pub(crate) struct ErrorsConfig {
    #[serde(rename = "404")]
    pub(crate) custom_404_page: Option<ErrorConfig>,
}

#[derive(Deserialize, Eq, PartialEq, Debug, Default, Clone)]
pub(crate) struct ErrorConfig {
    pub(crate) file_path: PathBuf,
    pub(crate) status: Option<u16>,
}

#[derive(Deserialize, Eq, PartialEq, Debug, Default)]
pub(crate) struct Header {
    pub(crate) path_matcher: String,
    pub(crate) key: String,
    pub(crate) value: String,
}

fn deserialize_headers<'de, D>(d: D) -> Result<Option<Vec<Header>>, D::Error>
where
    D: Deserializer<'de>,
{
    let deserialized = d.deserialize_map(HeadersVisitor)?;

    if deserialized.is_empty() {
        Ok(None)
    } else {
        Ok(Some(deserialized))
    }
}

struct HeadersVisitor;

impl<'de> Visitor<'de> for HeadersVisitor {
    type Value = Vec<Header>;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "a Heroku HTTP header map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut result = vec![];
        while let Some((key, value)) = map.next_entry::<String, BTreeMap<String, String>>()? {
            for (header_key, header_value) in value {
                result.push(Header {
                    path_matcher: key.clone(),
                    key: header_key,
                    value: header_value,
                });
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml::toml;

    #[test]
    fn custom_errors() {
        let toml_config = toml! {
            [errors]
            404.file_path = "error-404.html"
        };

        let parsed_config = toml_config.try_into::<HerokuWebServerConfig>().unwrap();
        assert_eq!(parsed_config.root, None);
        assert_eq!(parsed_config.index, None);
        assert_eq!(parsed_config.headers, None);
        assert_eq!(
            parsed_config.errors,
            Some(ErrorsConfig {
                custom_404_page: Some(ErrorConfig {
                    file_path: PathBuf::from("error-404.html"),
                    status: None,
                })
            })
        );
    }

    #[test]
    fn custom_root() {
        let toml_config = toml! {
            root = "files/web"
        };

        let parsed_config = toml_config.try_into::<HerokuWebServerConfig>().unwrap();
        assert_eq!(parsed_config.root, Some(PathBuf::from("files/web")));
        assert_eq!(parsed_config.index, None);
        assert_eq!(parsed_config.headers, None);
        assert_eq!(parsed_config.errors, None);
    }

    #[test]
    fn custom_index() {
        let toml_config = toml! {
            index = "main.html"
        };

        let parsed_config = toml_config.try_into::<HerokuWebServerConfig>().unwrap();
        assert_eq!(parsed_config.root, None);
        assert_eq!(parsed_config.index, Some("main.html".to_string()));
        assert_eq!(parsed_config.headers, None);
        assert_eq!(parsed_config.errors, None);
    }

    #[test]
    fn custom_headers() {
        let toml_config = toml! {
            [headers]
            "*".X-Global = "Hello"
            "/".X-Only-Default = "Hiii"
            "*.html".X-Only-HTML = "Hi"
            "/images/*".X-Only-Images = "HAI"
        };

        let parsed_config = toml_config.try_into::<HerokuWebServerConfig>().unwrap();
        assert_eq!(parsed_config.root, None);
        assert_eq!(parsed_config.index, None);
        assert_eq!(parsed_config.errors, None);

        assert_eq!(
            parsed_config.headers,
            Some(vec![
                Header {
                    path_matcher: String::from("*"),
                    key: String::from("X-Global"),
                    value: String::from("Hello")
                },
                Header {
                    path_matcher: String::from("/"),
                    key: String::from("X-Only-Default"),
                    value: String::from("Hiii")
                },
                Header {
                    path_matcher: String::from("*.html"),
                    key: String::from("X-Only-HTML"),
                    value: String::from("Hi")
                },
                Header {
                    path_matcher: String::from("/images/*"),
                    key: String::from("X-Only-Images"),
                    value: String::from("HAI")
                },
            ])
        );
    }
}
