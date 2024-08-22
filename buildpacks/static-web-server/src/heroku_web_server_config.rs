use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;
use std::fmt::Formatter;
use std::path::PathBuf;

#[derive(Deserialize, Default)]
pub(crate) struct HerokuWebServerConfig {
    pub(crate) root: Option<PathBuf>,
    pub(crate) errors: Option<ErrorsConfig>,
    #[serde(default, deserialize_with = "deserialize_headers")]
    pub(crate) headers: Vec<Header>,
}

#[derive(Deserialize, Eq, PartialEq, Debug, Default)]
pub(crate) struct ErrorsConfig {
    #[serde(rename = "404")]
    pub(crate) custom_404_page: Option<PathBuf>,
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub(crate) enum HeaderPathMatcher {
    Global,
    Path(String),
}

#[derive(Eq, PartialEq, Debug)]
pub(crate) struct Header {
    pub(crate) path_matcher: HeaderPathMatcher,
    pub(crate) key: String,
    pub(crate) value: String,
}

fn deserialize_headers<'de, D>(d: D) -> Result<Vec<Header>, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_map(HeadersVisitor)
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
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrMap {
            String(String),
            Map(
                // Ensure key/value order is maintained by using a b-tree
                BTreeMap<String, String>,
            ),
        }

        let mut result = vec![];
        while let Some((key, value)) = map.next_entry::<String, StringOrMap>()? {
            match value {
                StringOrMap::String(string) => result.push(Header {
                    path_matcher: HeaderPathMatcher::Global,
                    key,
                    value: string,
                }),
                StringOrMap::Map(key_values) => {
                    for (header_key, header_value) in key_values {
                        result.push(Header {
                            path_matcher: HeaderPathMatcher::Path(key.clone()),
                            key: header_key,
                            value: header_value,
                        });
                    }
                }
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
            404 = "public/error-404.html"
        };

        let parsed_config = toml_config.try_into::<HerokuWebServerConfig>().unwrap();
        assert_eq!(parsed_config.root, None);
        assert!(parsed_config.headers.is_empty());
        assert_eq!(
            parsed_config.errors,
            Some(ErrorsConfig {
                custom_404_page: Some(PathBuf::from("public/error-404.html"))
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
        assert!(parsed_config.headers.is_empty());
        assert_eq!(parsed_config.errors, None);
    }

    #[test]
    fn custom_headers() {
        let toml_config = toml! {
            [headers]
            X-Global = "Hello"
            "/".X-Only-Default = "Hiii"
            "*.html".X-Only-HTML = "Hi"
            "/images/*".X-Only-Images = "HAI"
        };

        let parsed_config = toml_config.try_into::<HerokuWebServerConfig>().unwrap();
        assert_eq!(parsed_config.root, None);
        assert_eq!(parsed_config.errors, None);

        assert_eq!(
            parsed_config.headers,
            vec![
                Header {
                    path_matcher: HeaderPathMatcher::Global,
                    key: String::from("X-Global"),
                    value: String::from("Hello")
                },
                Header {
                    path_matcher: HeaderPathMatcher::Path(String::from("/")),
                    key: String::from("X-Only-Default"),
                    value: String::from("Hiii")
                },
                Header {
                    path_matcher: HeaderPathMatcher::Path(String::from("*.html")),
                    key: String::from("X-Only-HTML"),
                    value: String::from("Hi")
                },
                Header {
                    path_matcher: HeaderPathMatcher::Path(String::from("/images/*")),
                    key: String::from("X-Only-Images"),
                    value: String::from("HAI")
                },
            ]
        );
    }
}
