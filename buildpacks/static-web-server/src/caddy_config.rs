use crate::heroku_web_server_config::{
    ErrorsConfig, HerokuWebServerConfig, PathMatchedHeader, DEFAULT_DOC_INDEX, DEFAULT_DOC_ROOT,
};
use crate::o11y::*;
use crate::StaticWebServerBuildpackError;
use indexmap::IndexMap;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;

/// Transforms the given [`HerokuWebServerConfig`] into an equivalent Caddy JSON configuration.
/// Keeping this as a single function, because many lines are just the JSON itself being assembled.
#[allow(clippy::too_many_lines)]
pub(crate) fn caddy_json_config(
    config: &HerokuWebServerConfig,
) -> Result<serde_json::Value, StaticWebServerBuildpackError> {
    let mut routes = vec![];

    // Header routes come first so headers will be added to any response down the chain.
    if let Some(ref headers) = config.headers {
        tracing::info!({ CONFIG_RESPONSE_HEADERS_ENABLED } = true, "config");
        routes.extend(generate_response_headers_routes(headers));
    }

    let doc_root = config
        .root
        .clone()
        .map_or(String::from(DEFAULT_DOC_ROOT), |path_buf| {
            String::from(path_buf.to_string_lossy())
        });

    let doc_index = config
        .index
        .clone()
        .unwrap_or(String::from(DEFAULT_DOC_INDEX));

    let mut static_file_handlers = vec![];

    if config
        .caddy_server_opts
        .as_ref()
        .is_some_and(|v| v.basic_auth.is_some_and(|vv| vv))
    {
        tracing::info!({ CONFIG_CADDY_SERVER_OPTS_BASIC_AUTH } = true, "config");
        static_file_handlers.push(json!(
        {
            "handler": "subroute",
            "routes": [{
                "match": [{
                    "expression": {
                        "expr": "{env.WEB_BASIC_AUTH_DISABLED} != 'true'",
                        "name": "basic_auth_required"
                    }
                }],
                "handle": [{
                    "handler": "authentication",
                    "providers": {
                        "http_basic": {
                            "accounts": [{
                                "username": "{env.WEB_BASIC_AUTH_USERNAME}",
                                "password": "{env.WEB_BASIC_AUTH_PASSWORD_BCRYPT}"
                            }],
                            "realm": "Restricted"
                        }
                    }
                }]
            }],
        }));
    }

    static_file_handlers.push(json!(
    {
        "handler": "encode",
        "encodings": {
            "zstd": { "level": "default" },
            "gzip": { "level": 6 }
        },
        "prefer": ["zstd", "gzip"]
    }));

    generate_static_response_handlers(config, &mut static_file_handlers)?;

    if config
        .caddy_server_opts
        .as_ref()
        .is_some_and(|v| v.templates.is_some_and(|vv| vv))
    {
        tracing::info!({ CONFIG_CADDY_SERVER_OPTS_TEMPLATES } = true, "config");
        static_file_handlers.push(json!(
        {
            "handler": "templates",
        }));
    }

    if config
        .caddy_server_opts
        .as_ref()
        .is_some_and(|v| v.clean_urls.is_some_and(|vv| vv))
    {
        tracing::info!({ CONFIG_CADDY_SERVER_OPTS_CLEAN_URLS } = true, "config");
        static_file_handlers.push(json!(
        {
            "handler": "subroute",
            "routes": [{
                "match": [{
                    "file": {
                        "root": doc_root,
                        "try_files": [
                            "{http.request.uri.path}",
                            "{http.request.uri.path}.html",
                            "{http.request.uri.path}/",
                        ]
                    }
                }],
                "handle": [{
                    "handler": "rewrite",
                    "uri": "{http.matchers.file.relative}",
                }]
            }],
        }));
    }

    static_file_handlers.push(json!(
    {
        "handler": "file_server",
        "root": doc_root,
        "index_names": vec![&doc_index],
        "pass_thru": true,
    }));

    routes.push(json!({
        "handle": static_file_handlers
    }));

    routes.push(generate_error_404_route(
        &doc_root,
        &doc_index,
        config.errors.as_ref(),
    ));

    let mut server_logs_config = json!(null);
    let caddy_access_logs_config = config
        .caddy_server_opts
        .as_ref()
        .and_then(|v| v.access_logs.as_ref());
    if caddy_access_logs_config.is_some_and(|vv| vv.enabled.is_some_and(|vvv| vvv)) {
        tracing::info!({ CONFIG_CADDY_SERVER_OPTS_ACCESS_LOGS } = true, "config");
        server_logs_config = json!({
            "default_logger_name": "public"
        });
    }

    Ok(json!({
        "apps": {
            "http": {
                "servers": {
                    "public": {
                        "listen": [":{env.PORT}"],
                        "logs": server_logs_config,
                        "routes": routes
                    }
                }
            }
        },
        "logging": {
            "sink": {
                "writer": {
                    "output": "stderr"
                }
            },
            "logs": {
                "default": {
                    "writer": {
                        "output": "stdout"
                    },
                    "encoder": {
                        "format": "json"
                    },
                    "exclude": [
                        "http.log.access.public"
                    ]
                },
                "public": {
                    "writer": {
                        "output": "stdout"
                    },
                    "encoder": {
                        "format": "json"
                    },
                    "sampling": {
                        "interval": caddy_access_logs_config.map_or(0, |v| v.sampling_interval.unwrap_or(0)),
                        "first": caddy_access_logs_config.map_or(0, |v| v.sampling_first.unwrap_or(0)),
                        "thereafter": caddy_access_logs_config.map_or(0, |v| v.sampling_thereafter.unwrap_or(0))
                    }
                }
            }
        }
    }))
}

fn generate_static_response_handlers(
    config: &HerokuWebServerConfig,
    static_file_handlers: &mut Vec<serde_json::Value>,
) -> Result<(), StaticWebServerBuildpackError> {
    if let Some(static_responses) = config
        .caddy_server_opts
        .as_ref()
        .and_then(|v| v.static_responses.clone())
    {
        tracing::info!(
            { CONFIG_CADDY_SERVER_OPTS_STATIC_RESPONSES } = true,
            "config"
        );
        for static_response in static_responses {
            // Validate that at least one matcher is set
            if static_response.host_matcher.is_none() && static_response.path_matcher.is_none() {
                return Err(StaticWebServerBuildpackError::ConfigurationConstraint(
                    "host_matcher or path_matcher must be set for caddy_server_opts.static_responses"
                        .to_string(),
                ));
            }

            let mut match_array = vec![];

            if let Some(host_matcher) = static_response.host_matcher {
                let mut host_match = serde_json::Map::new();
                host_match.insert("host".to_string(), json!(vec![host_matcher]));
                match_array.push(serde_json::Value::Object(host_match));
            }
            if let Some(path_matcher) = static_response.path_matcher {
                let mut path_match = serde_json::Map::new();
                path_match.insert("path".to_string(), json!(vec![path_matcher]));
                match_array.push(serde_json::Value::Object(path_match));
            }

            let headers = static_response.headers.map(|headers_vec| {
                headers_vec
                    .into_iter()
                    .map(|header| (header.key, vec![header.value]))
                    .collect::<HashMap<_, _>>()
            });

            let static_response_handler = StaticResponseHandler {
                handler: "static_response".to_string(),
                status_code: static_response.status.unwrap_or(200),
                headers,
                body: static_response.body,
            };
            let static_response_handler_json = serde_json::to_value(static_response_handler)
                .expect("StaticResponseHandler should serialize to JSON");

            static_file_handlers.push(json!(
            {
                "handler": "subroute",
                "routes": [{
                    "match": match_array,
                    "handle": [static_response_handler_json],
                    "terminal": true
                }],

            }));
        }
    }
    Ok(())
}

fn generate_response_headers_routes(headers: &Vec<PathMatchedHeader>) -> Vec<serde_json::Value> {
    // Group headers with the same matcher while preserving the order of the matchers
    // by "when-first-seen".
    let mut groups = IndexMap::<String, Vec<&PathMatchedHeader>>::new();
    for header in headers {
        if let Some(headers) = groups.get_mut(&header.path_matcher) {
            headers.push(header);
        } else {
            groups.insert(header.path_matcher.clone(), vec![header]);
        }
    }

    groups
        .into_iter()
        .map(|(matcher, headers)| {
            let headers_as_hash_map = headers
                .into_iter()
                .map(|header| (header.key.clone(), vec![header.value.clone()]))
                .collect::<HashMap<_, _>>();

            json!({
                "match": [{
                    "path": vec![matcher]
                }],
                "handle": [{
                    "handler": "headers",
                    "response": {
                        "set": headers_as_hash_map
                    }
                }]
            })
        })
        .collect()
}

fn generate_error_404_route(
    doc_root: &str,
    doc_index: &str,
    errors: Option<&ErrorsConfig>,
) -> serde_json::Value {
    let error_config = errors.and_then(|errors| errors.custom_404_page.as_ref());

    let not_found_response_handlers = error_config
        .map(|error_config| {
            let status_code = error_config.status.unwrap_or(404).to_string();
            tracing::info!(
                { CONFIG_ERROR_404_FILE_PATH } =
                    error_config.file_path.to_string_lossy().to_string(),
                "config"
            );
            tracing::info!({ CONFIG_ERROR_404_STATUS_CODE } = status_code, "config");

            json!([
                {
                    "handler": "rewrite",
                    "uri": error_config.file_path,
                },
                {
                    "handler": "file_server",
                    "root": doc_root,
                    "status_code": status_code,
                    "index_names": vec![doc_index],
                    "pass_thru": false
                }
            ])
        })
        .unwrap_or(json!([{
            "handler": "static_response",
            "status_code": "404",
            "headers": {
                "Content-Type": ["text/html"]
            },
            "body": DEFAULT_404_HTML
        }]));

    json!({
        "handle": not_found_response_handlers
    })
}

const DEFAULT_404_HTML: &str = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="utf-8">
        <title>404 Not Found</title>
    </head>
    <body>
        <h1>404 Not Found</h1>
    </body>
    </html>
"#;

#[derive(Serialize)]
struct StaticResponseHandler {
    handler: String,
    status_code: u16,
    headers: Option<HashMap<String, Vec<String>>>,
    body: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroku_web_server_config::{
        CaddyServerOpts, CaddyStaticResponseConfig, ErrorConfig, ErrorsConfig, Header,
    };
    use crate::StaticWebServerBuildpackError;
    use std::path::PathBuf;

    #[test]
    fn generates_matched_response_headers_routes() {
        let heroku_config = HerokuWebServerConfig {
            headers: Some(vec![
                PathMatchedHeader {
                    path_matcher: String::from("*"),
                    key: String::from("X-Foo"),
                    value: String::from("Bar"),
                },
                PathMatchedHeader {
                    path_matcher: String::from("*.html"),
                    key: String::from("X-Baz"),
                    value: String::from("Buz"),
                },
                PathMatchedHeader {
                    path_matcher: String::from("*"),
                    key: String::from("X-Zuu"),
                    value: String::from("Zem"),
                },
            ]),
            ..HerokuWebServerConfig::default()
        };

        let routes = generate_response_headers_routes(&heroku_config.headers.unwrap());

        assert_eq!(
            routes,
            vec![
                json!({"handle":[{"handler":"headers","response":{"set":{"X-Foo":["Bar"],"X-Zuu":["Zem"]}}}],"match":[{"path":["*"]}]}),
                json!({"handle":[{"handler":"headers","response":{"set":{"X-Baz":["Buz"]}}}],"match":[{"path":["*.html"]}]})
            ]
        );
    }

    #[test]
    fn generates_global_response_headers_routes() {
        let heroku_config = HerokuWebServerConfig {
            headers: Some(vec![PathMatchedHeader {
                path_matcher: String::from("*"),
                key: String::from("X-Foo"),
                value: String::from("Bar"),
            }]),
            ..HerokuWebServerConfig::default()
        };

        let routes = generate_response_headers_routes(&heroku_config.headers.unwrap());

        assert_eq!(
            routes,
            vec![
                json!({"handle": [{"handler":"headers","response":{"set":{"X-Foo":["Bar"]}}}],"match":[{"path":["*"]}]})
            ]
        );
    }

    #[test]
    fn generates_custom_404_error_route() {
        let doc_root = String::from("tests/fixtures/custom_errors/public");
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

        let route = generate_error_404_route(&doc_root, &doc_index, heroku_config.errors.as_ref());

        assert_eq!(
            route,
            json!({"handle":[{"handler":"rewrite","uri":"error-404.html"},{"handler":"file_server","index_names":["index.html"],"pass_thru":false,"root":"tests/fixtures/custom_errors/public","status_code":"404"}]})
        );
    }

    #[test]
    fn generates_custom_404_to_200_error_route() {
        let doc_root = String::from("tests/fixtures/client_side_routing/public");
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

        let route = generate_error_404_route(&doc_root, &doc_index, heroku_config.errors.as_ref());

        assert_eq!(
            route,
            json!({"handle":[{"handler":"rewrite","uri":"index.html"},{"handler":"file_server","index_names":["index.html"],"pass_thru":false,"root":"tests/fixtures/client_side_routing/public","status_code":"200"}]})
        );
    }

    #[test]
    fn generates_static_response_handlers() {
        let heroku_config = HerokuWebServerConfig {
            caddy_server_opts: Some(CaddyServerOpts {
                static_responses: Some(vec![
                    CaddyStaticResponseConfig {
                        expression_matcher: None,
                        host_matcher: Some("original.example.com".to_string()),
                        path_matcher: None,
                        status: Some(301),
                        headers: Some(vec![
                            Header {
                                key: "Location".to_string(),
                                value: "https://new.example.com{http.request.uri}".to_string(),
                            },
                            Header {
                                key: "X-Redirected-From".to_string(),
                                value: "original.example.com".to_string(),
                            },
                        ]),
                        body: None,
                    },
                    CaddyStaticResponseConfig {
                        expression_matcher: None,
                        host_matcher: Some("original.example.com".to_string()),
                        path_matcher: Some("/blog/*".to_string()),
                        status: Some(301),
                        headers: Some(vec![Header {
                            key: "Location".to_string(),
                            value:
                                "https://{http.request.host}/new-blog/{http.request.uri.path.file}"
                                    .to_string(),
                        }]),
                        body: None,
                    },
                    CaddyStaticResponseConfig {
                        expression_matcher: None,
                        host_matcher: None,
                        path_matcher: Some("/api/*".to_string()),
                        status: Some(500),
                        headers: Some(vec![Header {
                            key: "Content-Type".to_string(),
                            value: "application/json".to_string(),
                        }]),
                        body: Some(r#"{"error":"Service not available"}"#.to_string()),
                    },
                ]),
                ..CaddyServerOpts::default()
            }),
            ..HerokuWebServerConfig::default()
        };

        let mut handlers = vec![];
        generate_static_response_handlers(&heroku_config, &mut handlers).unwrap();

        assert_eq!(handlers.len(), 3);

        assert_eq!(
            handlers[0],
            json!({
                "handler": "subroute",
                "routes": [{
                    "match": [{"host": ["original.example.com"]}],
                    "handle": [{
                        "handler": "static_response",
                        "status_code": 301,
                        "headers": {
                            "Location": ["https://new.example.com{http.request.uri}"],
                            "X-Redirected-From": ["original.example.com"]
                        },
                        "body": null
                    }],
                    "terminal": true
                }]
            })
        );

        assert_eq!(
            handlers[1],
            json!({
                "handler": "subroute",
                "routes": [{
                    "match": [{"host": ["original.example.com"]}, {"path": ["/blog/*"]}],
                    "handle": [{
                        "handler": "static_response",
                        "status_code": 301,
                        "headers": {"Location": ["https://{http.request.host}/new-blog/{http.request.uri.path.file}"]},
                        "body": null
                    }],
                    "terminal": true
                }]
            })
        );

        assert_eq!(
            handlers[2],
            json!({
                "handler": "subroute",
                "routes": [{
                    "match": [{"path": ["/api/*"]}],
                    "handle": [{
                        "handler": "static_response",
                        "status_code": 500,
                        "headers": {"Content-Type": ["application/json"]},
                        "body": r#"{"error":"Service not available"}"#
                    }],
                    "terminal": true
                }]
            })
        );
    }

    #[test]
    fn generates_static_response_handlers_error_when_no_matchers() {
        let heroku_config = HerokuWebServerConfig {
            caddy_server_opts: Some(CaddyServerOpts {
                static_responses: Some(vec![CaddyStaticResponseConfig {
                    expression_matcher: None,
                    host_matcher: None,
                    path_matcher: None,
                    status: Some(200),
                    headers: None,
                    body: Some("test".to_string()),
                }]),
                ..CaddyServerOpts::default()
            }),
            ..HerokuWebServerConfig::default()
        };

        let mut handlers = vec![];
        let result = generate_static_response_handlers(&heroku_config, &mut handlers);

        assert!(result.is_err());
        if let Err(StaticWebServerBuildpackError::ConfigurationConstraint(msg)) = result {
            assert_eq!(
                msg,
                "host_matcher or path_matcher must be set for caddy_server_opts.static_responses"
            );
        } else {
            panic!("Expected ConfigurationConstraint error");
        }
    }
}
