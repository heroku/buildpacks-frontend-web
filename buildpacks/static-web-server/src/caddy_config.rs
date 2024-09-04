use crate::errors::StaticWebServerBuildpackError;
use crate::errors::StaticWebServerBuildpackError::CannotReadCustom404File;
use crate::heroku_web_server_config::{ErrorsConfig, Header, HerokuWebServerConfig, DEFAULT_DOC_ROOT, DEFAULT_DOC_INDEX};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CaddyConfig {
    pub(crate) apps: CaddyConfigApps,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CaddyConfigApps {
    pub(crate) http: CaddyConfigAppHTTP,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CaddyConfigAppHTTP {
    pub(crate) servers: CaddyConfigHTTPServers,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CaddyConfigHTTPServers {
    pub(crate) public: CaddyConfigHTTPServerPublic,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CaddyConfigHTTPServerPublic {
    pub(crate) listen: Vec<String>,
    pub(crate) routes: Vec<CaddyHTTPServerRoute>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CaddyHTTPServerRoute {
    pub(crate) r#match: Option<Vec<CaddyHTTPServerRouteMatcher>>,
    pub(crate) handle: Vec<CaddyHTTPServerRouteHandler>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CaddyHTTPServerErrors {
    pub(crate) routes: Vec<CaddyHTTPServerRoute>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum CaddyHTTPServerRouteMatcher {
    // https://caddyserver.com/docs/json/apps/http/servers/routes/match/path/
    Path(MatchPath),
}

// https://caddyserver.com/docs/json/apps/http/servers/routes/match/path/
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct MatchPath {
    pub(crate) path: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum CaddyHTTPServerRouteHandler {
    FileServer(FileServer),
    Headers(Headers),
    StaticResponse(StaticResponse),
}

// https://caddyserver.com/docs/json/apps/http/servers/routes/handle/file_server/
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct FileServer {
    pub(crate) handler: String,
    pub(crate) root: String,
    pub(crate) index_names: Vec<String>,
    pub(crate) pass_thru: bool,
}

// https://caddyserver.com/docs/json/apps/http/servers/routes/handle/headers/
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Headers {
    pub(crate) handler: String,
    pub(crate) response: HeadersResponse,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct HeadersResponse {
    pub(crate) set: serde_json::Map<String, serde_json::Value>,
    pub(crate) deferred: bool,
}

// https://caddyserver.com/docs/json/apps/http/servers/routes/handle/static_response/
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct StaticResponse {
    pub(crate) handler: String,
    pub(crate) status_code: String,
    pub(crate) headers: Option<serde_json::Map<String, serde_json::Value>>,
    pub(crate) body: String,
}

impl TryFrom<HerokuWebServerConfig> for CaddyConfig {
    type Error = StaticWebServerBuildpackError;

    fn try_from(value: HerokuWebServerConfig) -> Result<Self, Self::Error> {
        let mut routes = vec![];

        // Header routes come first so headers will be added to any response down the chain.
        if value.headers.is_some() {
            let h = value.headers.unwrap_or_default();
            routes.extend(generate_response_headers_routes(&h));
        }

        let doc_root = value
            .root
            .clone()
            .unwrap_or(PathBuf::from(DEFAULT_DOC_ROOT));

        let doc_index = value
            .index
            .clone()
            .unwrap_or(String::from(DEFAULT_DOC_INDEX));

        // Default router is just the static file server.
        // This vector will contain all routes in order of request processing,
        // while response processing is reverse direction.
        routes.push(CaddyHTTPServerRoute {
            r#match: None,
            handle: vec![CaddyHTTPServerRouteHandler::FileServer(FileServer {
                handler: "file_server".to_owned(),
                root: doc_root
                    .to_string_lossy()
                    .to_string(),
                index_names: vec![doc_index],
                // Any not found request paths continue to the next handler.
                pass_thru: true,
            })],
        });

        routes.push(generate_error_404_route(&doc_root, &value.errors)?);

        // Assemble into the caddy.json structure
        // https://caddyserver.com/docs/json/
        Ok(CaddyConfig {
            apps: CaddyConfigApps {
                http: CaddyConfigAppHTTP {
                    servers: CaddyConfigHTTPServers {
                        public: CaddyConfigHTTPServerPublic {
                            listen: vec![":{env.PORT}".to_owned()],
                            routes,
                        },
                    },
                },
            },
        })
    }
}

fn generate_response_headers_routes(headers: &Vec<Header>) -> Vec<CaddyHTTPServerRoute> {
    // Group headers with the same matcher while preserving the order of the matchers
    // by "when-first-seen".
    let mut groups = IndexMap::<String, Vec<&Header>>::new();
    for header in headers {
        if let Some(headers) = groups.get_mut(&header.path_matcher) {
            headers.push(header);
        } else {
            groups.insert(header.path_matcher.clone(), vec![header]);
        }
    }

    groups
        .into_iter()
        .map(|(matcher, headers)| CaddyHTTPServerRoute {
            r#match: Some(vec![CaddyHTTPServerRouteMatcher::Path(MatchPath {
                path: vec![matcher],
            })]),
            handle: vec![CaddyHTTPServerRouteHandler::Headers(Headers {
                handler: "headers".to_owned(),
                response: HeadersResponse {
                    set: headers
                        .into_iter()
                        .map(|header| {
                            (
                                header.key.clone(),
                                serde_json::Value::Array(vec![serde_json::Value::String(
                                    header.value.clone(),
                                )]),
                            )
                        })
                        .collect::<serde_json::Map<String, serde_json::Value>>(),
                    deferred: true,
                },
            })],
        })
        .collect()
}

fn generate_error_404_route(
    doc_root: &PathBuf,
    errors: &Option<ErrorsConfig>,
) -> Result<CaddyHTTPServerRoute, StaticWebServerBuildpackError> {
    let error_config = errors
        .as_ref()
        .and_then(|errors| errors.custom_404_page.clone());

    let status_code = error_config
        .as_ref()
        .map_or(404, |error| error.status.unwrap_or(404))
        .to_string();

    let not_found_response_content = error_config
        .as_ref()
        .map(|error| error.file_path.clone())
        .map_or(
            {
                let default = r#"<!DOCTYPE html>
                <html lang="en">
                  <head>
                    <meta charset="utf-8">
                    <title>404 Not Found</title>
                  </head>
                  <body>
                    <h1>404 Not Found</h1>
                  </body>
                </html>"#;

                Ok(String::from(default))
            },
            |file| {
                let content = fs::read_to_string(doc_root.join(file)).map_err(CannotReadCustom404File)?;
                if status_code == "404" {
                    Ok(content)
                } else {
                    Ok(format!("<!-- This is actually a Not Found response from the static web server. -->\n{content}"))
                }
            },
        )?;

    Ok(CaddyHTTPServerRoute {
        r#match: None,
        handle: vec![CaddyHTTPServerRouteHandler::StaticResponse(
            StaticResponse {
                handler: "static_response".to_owned(),
                status_code: status_code.clone(),
                headers: Some({
                    let mut h = serde_json::Map::new();
                    h.insert(
                        "Content-Type".to_string(),
                        serde_json::Value::Array(vec![serde_json::Value::String(
                            "text/html".to_string(),
                        )]),
                    );
                    h
                }),
                body: not_found_response_content.clone(),
            },
        )],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroku_web_server_config::{ErrorConfig, ErrorsConfig};
    use libherokubuildpack::log::log_info;
    use std::path::PathBuf;

    #[test]
    fn generates_matched_response_headers_routes() {
        let heroku_config = HerokuWebServerConfig {
            headers: Some(vec![
                Header {
                    path_matcher: String::from("*"),
                    key: String::from("X-Foo"),
                    value: String::from("Bar"),
                },
                Header {
                    path_matcher: String::from("*.html"),
                    key: String::from("X-Baz"),
                    value: String::from("Buz"),
                },
                Header {
                    path_matcher: String::from("*"),
                    key: String::from("X-Zuu"),
                    value: String::from("Zem"),
                },
            ]),
            ..HerokuWebServerConfig::default()
        };

        let routes = generate_response_headers_routes(&heroku_config.headers.unwrap());

        assert_eq!(routes.len(), 2, "should generate two routes");

        // First route
        let generated_route = &routes[0];
        let generated_match = generated_route
            .r#match
            .as_ref()
            .expect("route should contain match");
        let CaddyHTTPServerRouteMatcher::Path(expected_matcher) = &generated_match[0];
        assert_eq!(expected_matcher.path[0], "*", "should have match path *");

        let CaddyHTTPServerRouteHandler::Headers(generated_handler) = &generated_route.handle[0]
        else {
            unreachable!()
        };

        assert_eq!(
            generated_handler.handler, "headers",
            "should be a headers route"
        );

        let generated_headers_to_set = &generated_handler.response.set;
        assert!(
            generated_headers_to_set.contains_key("X-Foo"),
            "should contain header X-Foo"
        );

        let expected_key = "X-Foo";
        let expected_value =
            serde_json::Value::Array(vec![serde_json::Value::String("Bar".to_string())]);
        assert_eq!(
            generated_headers_to_set.get(expected_key),
            Some(&expected_value),
            "should contain header value Bar"
        );

        let expected_key = "X-Zuu";
        let expected_value =
            serde_json::Value::Array(vec![serde_json::Value::String("Zem".to_string())]);
        assert_eq!(
            generated_headers_to_set.get(expected_key),
            Some(&expected_value),
            "should contain header value Zem"
        );

        // Second route
        let generated_route = &routes[1];
        let generated_match = generated_route
            .r#match
            .as_ref()
            .expect("route should contain match");
        let CaddyHTTPServerRouteMatcher::Path(expected_matcher) = &generated_match[0];
        assert_eq!(
            expected_matcher.path[0], "*.html",
            "should have match path *.html"
        );

        let CaddyHTTPServerRouteHandler::Headers(generated_handler) = &generated_route.handle[0]
        else {
            unreachable!()
        };

        assert_eq!(
            generated_handler.handler, "headers",
            "should be a headers route"
        );

        let generated_headers_to_set = &generated_handler.response.set;
        assert!(
            generated_headers_to_set.contains_key("X-Baz"),
            "should contain header X-Baz"
        );
    }

    #[test]
    fn generates_global_response_headers_routes() {
        let heroku_config = HerokuWebServerConfig {
            headers: Some(vec![Header {
                path_matcher: String::from("*"),
                key: String::from("X-Foo"),
                value: String::from("Bar"),
            }]),
            ..HerokuWebServerConfig::default()
        };

        let routes = generate_response_headers_routes(&heroku_config.headers.unwrap());
        assert_eq!(routes.len(), 1, "should generate one route");

        let generated_route = &routes[0];
        let generated_match = generated_route
            .r#match
            .as_ref()
            .expect("route should contain match");

        let CaddyHTTPServerRouteMatcher::Path(expected_matcher) = &generated_match[0];

        assert_eq!(expected_matcher.path[0], "*", "should have match path *");

        let CaddyHTTPServerRouteHandler::Headers(generated_handler) = &generated_route.handle[0]
        else {
            unreachable!()
        };
        assert_eq!(
            generated_handler.handler, "headers",
            "should be a headers route"
        );

        let generated_headers_to_set = &generated_handler.response.set;
        assert!(
            generated_headers_to_set.contains_key("X-Foo"),
            "should contain header X-Foo"
        );

        let expected_key = "X-Foo";
        let expected_value =
            serde_json::Value::Array(vec![serde_json::Value::String("Bar".to_string())]);
        assert_eq!(
            generated_headers_to_set.get(expected_key),
            Some(&expected_value),
            "should contain header value Bar"
        );
    }

    #[test]
    fn generates_custom_404_error_route() {
        let doc_root = PathBuf::from("tests/fixtures/custom_errors/public");

        let heroku_config = HerokuWebServerConfig {
            errors: Some(ErrorsConfig {
                custom_404_page: Some(ErrorConfig {
                    file_path: PathBuf::from("error-404.html"),
                    status: None,
                }),
            }),
            ..HerokuWebServerConfig::default()
        };

        let routes = generate_error_404_route(&doc_root, &heroku_config.errors).unwrap();

        let CaddyHTTPServerRouteHandler::StaticResponse(generated_handler) = &routes.handle[0]
        else {
            unreachable!()
        };

        assert_eq!(
            generated_handler.handler, "static_response",
            "should be a static_response route"
        );

        let generated_status = &generated_handler.status_code;
        assert!(generated_status.eq("404"), "status_code should by 404");

        let generated_body = &generated_handler.body;
        assert!(
            generated_body.contains("Custom 404"),
            "body should contain Custom 404"
        );
    }

    #[test]
    fn generates_custom_404_to_200_error_route() {
        let doc_root = PathBuf::from("tests/fixtures/client_side_routing/public");

        let heroku_config = HerokuWebServerConfig {
            errors: Some(ErrorsConfig {
                custom_404_page: Some(ErrorConfig {
                    file_path: PathBuf::from(
                        "index.html",
                    ),
                    status: Some(200),
                }),
            }),
            ..HerokuWebServerConfig::default()
        };

        let routes = generate_error_404_route(&doc_root, &heroku_config.errors).unwrap();

        let CaddyHTTPServerRouteHandler::StaticResponse(generated_handler) = &routes.handle[0]
        else {
            unreachable!()
        };

        assert_eq!(
            generated_handler.handler, "static_response",
            "should be a static_response route"
        );

        let generated_status = &generated_handler.status_code;
        assert!(generated_status.eq("200"), "status_code should by 200");

        let generated_body = &generated_handler.body;
        assert!(
            generated_body.contains("Client Side Routing Test"),
            "body should contain Client Side Routing Test"
        );
    }

    #[test]
    fn missing_custom_404_error_file() {
        let doc_root = PathBuf::from(DEFAULT_DOC_ROOT);

        let heroku_config = HerokuWebServerConfig {
            errors: Some(ErrorsConfig {
                custom_404_page: Some(ErrorConfig {
                    file_path: PathBuf::from("non-existent-path"),
                    status: None,
                }),
            }),
            ..HerokuWebServerConfig::default()
        };

        match generate_error_404_route(&doc_root, &heroku_config.errors) {
            Ok(_) => {
                panic!("should fail to find custom 404 file");
            }
            Err(e) => {
                log_info(format!("Missing 404 file error: {e:?}"));
            }
        };
    }
}
