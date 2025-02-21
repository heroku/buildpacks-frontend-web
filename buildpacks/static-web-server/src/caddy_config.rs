use crate::errors::StaticWebServerBuildpackError;
use crate::heroku_web_server_config::{
    ErrorsConfig, Header, HerokuWebServerConfig, DEFAULT_DOC_INDEX, DEFAULT_DOC_ROOT,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
    Rewrite(Rewrite),
    StaticResponse(StaticResponse),
}

// https://caddyserver.com/docs/json/apps/http/servers/routes/handle/file_server/
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct FileServer {
    pub(crate) handler: String,
    pub(crate) root: String,
    pub(crate) index_names: Vec<String>,
    pub(crate) pass_thru: bool,
    pub(crate) status_code: Option<String>,
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

// https://caddyserver.com/docs/json/apps/http/servers/routes/handle/rewrite/
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Rewrite {
    pub(crate) handler: String,
    pub(crate) uri: Option<String>,
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
                root: doc_root.to_string_lossy().to_string(),
                index_names: vec![doc_index.clone()],
                pass_thru: true,
                status_code: None,
            })],
        });

        routes.push(generate_error_404_route(
            &doc_root,
            &doc_index,
            value.errors.as_ref(),
        )?);

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
    doc_root: &Path,
    doc_index: &str,
    errors: Option<&ErrorsConfig>,
) -> Result<CaddyHTTPServerRoute, StaticWebServerBuildpackError> {
    let error_config = errors.and_then(|errors| errors.custom_404_page.clone());

    let status_code = error_config
        .as_ref()
        .map_or(404, |error| error.status.unwrap_or(404))
        .to_string();

    let not_found_response_handlers = error_config
        .as_ref()
        .map(|error| error.file_path.clone())
        .map_or(
            {
                Ok(vec![CaddyHTTPServerRouteHandler::StaticResponse(
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
                        body: r#"<!DOCTYPE html>
                            <html lang="en">
                            <head>
                                <meta charset="utf-8">
                                <title>404 Not Found</title>
                            </head>
                            <body>
                                <h1>404 Not Found</h1>
                            </body>
                            </html>"#
                            .to_string(),
                    },
                )])
            },
            |file| {
                Ok(vec![
                    CaddyHTTPServerRouteHandler::Rewrite(Rewrite {
                        handler: "rewrite".to_owned(),
                        uri: Some(file.to_string_lossy().to_string()),
                    }),
                    CaddyHTTPServerRouteHandler::FileServer(FileServer {
                        handler: "file_server".to_owned(),
                        root: doc_root.to_string_lossy().to_string(),
                        status_code: Some(status_code.clone()),
                        index_names: vec![doc_index.to_string()],
                        pass_thru: false,
                    }),
                ])
            },
        )?;

    Ok(CaddyHTTPServerRoute {
        r#match: None,
        handle: not_found_response_handlers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroku_web_server_config::{ErrorConfig, ErrorsConfig};
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
        let doc_index = "index.html".to_string();

        let heroku_config = HerokuWebServerConfig {
            errors: Some(ErrorsConfig {
                custom_404_page: Some(ErrorConfig {
                    file_path: PathBuf::from("error-404.html"),
                    status: None,
                }),
            }),
            ..HerokuWebServerConfig::default()
        };

        let routes =
            generate_error_404_route(&doc_root, &doc_index, heroku_config.errors.as_ref()).unwrap();

        let CaddyHTTPServerRouteHandler::Rewrite(generated_rewrite_handler) = &routes.handle[0]
        else {
            unreachable!()
        };

        assert_eq!(
            generated_rewrite_handler.handler, "rewrite",
            "should be a rewrite handler"
        );

        assert_eq!(
            generated_rewrite_handler.uri,
            Some("error-404.html".to_string()),
            "should rewrite to error-404.html"
        );

        let CaddyHTTPServerRouteHandler::FileServer(generated_fs_handler) = &routes.handle[1]
        else {
            unreachable!()
        };

        assert_eq!(
            generated_fs_handler.handler, "file_server",
            "should be a file_server handler"
        );

        let generated_status = &generated_fs_handler.status_code;
        assert!(
            generated_status.eq(&Some("404".to_string())),
            "status_code should be 404"
        );
    }

    #[test]
    fn generates_custom_404_to_200_error_route() {
        let doc_root = PathBuf::from("tests/fixtures/client_side_routing/public");
        let doc_index = "index.html".to_string();

        let heroku_config = HerokuWebServerConfig {
            errors: Some(ErrorsConfig {
                custom_404_page: Some(ErrorConfig {
                    file_path: PathBuf::from("index.html"),
                    status: Some(200),
                }),
            }),
            ..HerokuWebServerConfig::default()
        };

        let routes =
            generate_error_404_route(&doc_root, &doc_index, heroku_config.errors.as_ref()).unwrap();

        let CaddyHTTPServerRouteHandler::Rewrite(generated_rewrite_handler) = &routes.handle[0]
        else {
            unreachable!()
        };

        assert_eq!(
            generated_rewrite_handler.handler, "rewrite",
            "should be a rewrite handler"
        );

        assert_eq!(
            generated_rewrite_handler.uri,
            Some("index.html".to_string()),
            "should rewrite to index.html"
        );

        let CaddyHTTPServerRouteHandler::FileServer(generated_fs_handler) = &routes.handle[1]
        else {
            unreachable!()
        };

        assert_eq!(
            generated_fs_handler.handler, "file_server",
            "should be a file_server handler"
        );

        let generated_status = &generated_fs_handler.status_code;
        assert!(
            generated_status.eq(&Some("200".to_string())),
            "status_code should be 200"
        );
    }
}
