use crate::heroku_web_server_config::{
    ErrorsConfig, Header, HerokuWebServerConfig, DEFAULT_DOC_INDEX, DEFAULT_DOC_ROOT,
};
use indexmap::IndexMap;
use serde_json::json;
use std::collections::HashMap;

/// Transforms the given [`HerokuWebServerConfig`] into an equivalent Caddy JSON configuration.
pub(crate) fn caddy_json_config(config: HerokuWebServerConfig) -> serde_json::Value {
    let mut routes = vec![];

    // Header routes come first so headers will be added to any response down the chain.
    if let Some(headers) = config.headers {
        routes.extend(generate_response_headers_routes(&headers));
    }

    let doc_root = config
        .root
        .map_or(String::from(DEFAULT_DOC_ROOT), |path_buf| {
            String::from(path_buf.to_string_lossy())
        });

    let doc_index = config
        .index
        .clone()
        .unwrap_or(String::from(DEFAULT_DOC_INDEX));

    let mut static_file_handlers = vec![];

    static_file_handlers.push(json!(
    {
        "handler": "encode",
        "encodings": {
            "zstd": { "level": "default" },
            "gzip": { "level": 6 }
        },
        "prefer": ["zstd", "gzip"]
    }));

    if config
        .caddy_server_opts
        .as_ref()
        .is_some_and(|v| v.templates.is_some_and(|vv| vv))
    {
        static_file_handlers.push(json!(
        {
            "handler": "templates",
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
        server_logs_config = json!({
            "default_logger_name": "public"
        });
    }

    json!({
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
    })
}

fn generate_response_headers_routes(headers: &Vec<Header>) -> Vec<serde_json::Value> {
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
            headers: Some(vec![Header {
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
}
