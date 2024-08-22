use std::fs;
use std::path::PathBuf;

use crate::caddy_config::*;
use crate::{StaticWebServerBuildpack, StaticWebServerBuildpackError};
use libcnb::data::layer_name;
use libcnb::layer::LayerRef;
use libcnb::read_toml_file;
use libcnb::{build::BuildContext, layer::UncachedLayerDefinition};
use libherokubuildpack::log::log_info;
use libherokubuildpack::toml::toml_select_value;
use serde::{Deserialize, Serialize};
use toml::toml;

const DEFAULT_DOC_ROOT: &str = "public";

pub(crate) fn config_web_server(
    context: &BuildContext<StaticWebServerBuildpack>,
) -> Result<LayerRef<StaticWebServerBuildpack, (), ()>, libcnb::Error<StaticWebServerBuildpackError>>
{
    let configuration_layer = context.uncached_layer(
        layer_name!("configuration"),
        UncachedLayerDefinition {
            build: false,
            launch: true,
        },
    )?;

    let project_toml: toml::Value = read_toml_file(context.app_dir.join("project.toml"))
        .unwrap_or_else(|_| {
            toml::Value::try_from(toml::Table::new())
                .expect("should return the TOML Value of an empty TOML Table")
        });

    let doc_root = toml_select_value(vec!["_", "metadata", "web-server", "root"], &project_toml)
        .and_then(toml::Value::as_str)
        .unwrap_or(DEFAULT_DOC_ROOT);

    // Default router is just the static file server.
    // This vector will contain all routes in order of request processing,
    // while response processing is reverse direction.
    let mut routes: Vec<CaddyHTTPServerRoute> = vec![CaddyHTTPServerRoute {
        r#match: None,
        handle: vec![CaddyHTTPServerRouteHandler::FileServer(FileServer {
            handler: "file_server".to_owned(),
            root: doc_root.to_string(),
            // Any not found request paths continue to the next handler.
            pass_thru: true,
        })],
    }];

    routes = generate_response_headers_routes(&project_toml, routes);
    routes = generate_error_404_route(&project_toml, &context.app_dir, routes)?;

    // Assemble into the caddy.json structure
    // https://caddyserver.com/docs/json/
    let caddy_config = CaddyConfig {
        apps: CaddyConfigApps {
            http: CaddyConfigAppHTTP {
                servers: CaddyConfigHTTPServers {
                    public: CaddyConfigHTTPServerPublic {
                        listen: vec![":{env.PORT}".to_owned()],
                        routes: routes,
                    },
                },
            },
        },
    };

    let caddy_config_json =
        serde_json::to_string(&caddy_config).map_err(StaticWebServerBuildpackError::JSON)?;

    log_info(format!("caddy.json {:?}", caddy_config_json));

    let config_path = configuration_layer.path().join("caddy.json");
    fs::write(&config_path, caddy_config_json).map_err(|e| {
        StaticWebServerBuildpackError::Message(format!(
            "{}, when writing config file {:?}",
            e, &config_path
        ))
    })?;

    Ok(configuration_layer)
}

fn generate_response_headers_routes(
    project_toml: &toml::Value,
    routes: Vec<CaddyHTTPServerRoute>,
) -> Vec<CaddyHTTPServerRoute> {
    let default_toml = toml::Table::new();
    let response_headers = toml_select_value(
        vec!["_", "metadata", "web-server", "headers"],
        &project_toml,
    )
    .and_then(toml::Value::as_table)
    .unwrap_or(&default_toml);
    let mut new_routes: Vec<CaddyHTTPServerRoute> = vec![];

    response_headers.iter().for_each(|(k, v)| {
        // Detect when header is defined without a path matcher (the value is not a table, probably a string)
        let headers_for_match = v.as_table().unwrap_or(&default_toml);
        // Default to * path matcher, when missing, otherwise use the current key as path matcher
        let header_match = if headers_for_match.is_empty() {
            vec![CaddyHTTPServerRouteMatcher::Path(MatchPath {
                path: vec!["*".to_string()],
            })]
        } else {
            vec![CaddyHTTPServerRouteMatcher::Path(MatchPath {
                path: vec![k.to_string()],
            })]
        };
        // When header is defined without path matcher, reset to configure header directly with key & value
        let header_values_to_config = if headers_for_match.is_empty() {
            let mut t = toml::Table::new();
            t.insert(k.to_string(), v.to_owned());
            t
        } else {
            headers_for_match.to_owned()
        };
        let mut header_values = serde_json::Map::new();
        header_values_to_config.iter().for_each(|(kk, vv)| {
            header_values.insert(
                kk.to_string(),
                serde_json::Value::Array(vec![serde_json::Value::String(
                    vv.as_str().unwrap_or_default().to_string(),
                )]),
            );
        });
        let new_route = CaddyHTTPServerRoute {
            r#match: Some(header_match),
            handle: vec![CaddyHTTPServerRouteHandler::Headers(Headers {
                handler: "headers".to_owned(),
                response: HeadersResponse {
                    set: header_values,
                    deferred: true,
                },
            })],
        };
        // Append each new route, maintaining order of TOML config
        new_routes.push(new_route);
    });
    // Prepend the new routes, so they come before existing routes (file server)
    new_routes.into_iter().chain(routes.into_iter()).collect()
}

fn generate_error_404_route(
    project_toml: &toml::Value,
    app_dir: &PathBuf,
    routes: Vec<CaddyHTTPServerRoute>,
) -> Result<Vec<CaddyHTTPServerRoute>, StaticWebServerBuildpackError> {
    let default_toml = toml::Table::new();
    let custom_404 = toml_select_value(
        vec!["_", "metadata", "web-server", "errors", "404"],
        &project_toml,
    )
    .and_then(toml::Value::as_str)
    .unwrap_or("");
    let response_body = if custom_404.is_empty() {
        r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>404 Not Found</title>
  </head>
  <body>
    <h1>404 Not Found</h1>
  </body>
</html>"#
            .to_string()
    } else {
        let custom_error_path = app_dir.join(custom_404);
        fs::read_to_string(&custom_error_path).map_err(|e| {
            StaticWebServerBuildpackError::Message(format!(
                "{}, when opening 404 error file {:?}",
                e, &custom_error_path
            ))
        })?
    };

    let new_routes = vec![CaddyHTTPServerRoute {
        r#match: None,
        handle: vec![CaddyHTTPServerRouteHandler::StaticResponse(
            StaticResponse {
                handler: "static_response".to_owned(),
                status_code: "404".to_string(),
                headers: Some((|| {
                    let mut h = serde_json::Map::new();
                    h.insert(
                        "Content-Type".to_string(),
                        serde_json::Value::Array(vec![serde_json::Value::String(
                            "text/html".to_string(),
                        )]),
                    );
                    h
                })()),
                body: response_body,
            },
        )],
    }];
    // Append the new routes, so they come after existing routes (file server)
    Ok(routes.into_iter().chain(new_routes.into_iter()).collect())
}

#[cfg(test)]
mod tests {
    use std::{env, path::PathBuf};

    use super::*;

    #[test]
    fn generates_matched_response_headers_routes() {
        let project_toml = toml::Value::Table(toml! {
            ["_".metadata.web-server.headers]
            "*".X-Foo = "Bar"
            "*.html".X-Baz = "Buz"
            "*".X-Zuu = "Zem"
        });
        let mut routes: Vec<CaddyHTTPServerRoute> = vec![];
        routes = generate_response_headers_routes(&project_toml, routes);
        assert!(routes.len() == 2, "should generate two routes");

        // First route
        let generated_route = &routes[0];
        let generated_match = generated_route
            .r#match
            .as_ref()
            .expect("route should contain match");
        let expected_matcher = if let CaddyHTTPServerRouteMatcher::Path(m) = &generated_match[0] {
            m
        } else {
            unreachable!()
        };
        assert!(expected_matcher.path[0] == "*", "should have match path *");

        let generated_handler =
            if let CaddyHTTPServerRouteHandler::Headers(h) = &generated_route.handle[0] {
                h
            } else {
                unreachable!()
            };
        assert!(
            generated_handler.handler == "headers",
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
        assert!(
            generated_headers_to_set.get(expected_key) == Some(&expected_value),
            "should contain header value Bar"
        );

        let expected_key = "X-Zuu";
        let expected_value =
            serde_json::Value::Array(vec![serde_json::Value::String("Zem".to_string())]);
        assert!(
            generated_headers_to_set.get(expected_key) == Some(&expected_value),
            "should contain header value Zem"
        );

        // Second route
        let generated_route = &routes[1];
        let generated_match = generated_route
            .r#match
            .as_ref()
            .expect("route should contain match");
        let expected_matcher = if let CaddyHTTPServerRouteMatcher::Path(m) = &generated_match[0] {
            m
        } else {
            unreachable!()
        };
        assert!(
            expected_matcher.path[0] == "*.html",
            "should have match path *.html"
        );

        let generated_handler =
            if let CaddyHTTPServerRouteHandler::Headers(h) = &generated_route.handle[0] {
                h
            } else {
                unreachable!()
            };
        assert!(
            generated_handler.handler == "headers",
            "should be a headers route"
        );

        let generated_headers_to_set = &generated_handler.response.set;
        assert!(
            generated_headers_to_set.contains_key("X-Baz"),
            "should contain header X-Baz"
        );

        let expected_key = "X-Baz";
        let expected_value =
            serde_json::Value::Array(vec![serde_json::Value::String("Buz".to_string())]);
    }

    #[test]
    fn generates_global_response_headers_routes() {
        let project_toml = toml::Value::Table(toml! {
            ["_".metadata.web-server.headers]
            X-Foo = "Bar"
        });
        let mut routes: Vec<CaddyHTTPServerRoute> = vec![];
        routes = generate_response_headers_routes(&project_toml, routes);
        assert!(routes.len() == 1, "should generate one route");

        let generated_route = &routes[0];
        let generated_match = generated_route
            .r#match
            .as_ref()
            .expect("route should contain match");
        let expected_matcher = if let CaddyHTTPServerRouteMatcher::Path(m) = &generated_match[0] {
            m
        } else {
            unreachable!()
        };
        assert!(expected_matcher.path[0] == "*", "should have match path *");

        let generated_handler =
            if let CaddyHTTPServerRouteHandler::Headers(h) = &generated_route.handle[0] {
                h
            } else {
                unreachable!()
            };
        assert!(
            generated_handler.handler == "headers",
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
        assert!(
            generated_headers_to_set.get(expected_key) == Some(&expected_value),
            "should contain header value Bar"
        );
    }

    #[test]
    fn generates_custom_404_error_route() {
        let project_toml = toml::Value::Table(toml! {
            ["_".metadata.web-server.errors]
            404 = "public/error-404.html"
        });
        let app_dir =
            PathBuf::from(env::current_dir().unwrap()).join("tests/fixtures/custom_errors");
        let mut routes: Vec<CaddyHTTPServerRoute> = vec![];
        routes = generate_error_404_route(&project_toml, &app_dir, routes).unwrap();
        assert!(routes.len() == 1, "should generate one route");

        let generated_route = &routes[0];
        let generated_handler =
            if let CaddyHTTPServerRouteHandler::StaticResponse(h) = &generated_route.handle[0] {
                h
            } else {
                unreachable!()
            };
        assert!(
            generated_handler.handler == "static_response",
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
    fn missing_custom_404_error_file() {
        let project_toml = toml::Value::Table(toml! {
            ["_".metadata.web-server.errors]
            404 = "non-existent-path"
        });
        let app_dir =
            PathBuf::from(env::current_dir().unwrap()).join("tests/fixtures/custom_errors");
        let mut routes: Vec<CaddyHTTPServerRoute> = vec![];
        match generate_error_404_route(&project_toml, &app_dir, routes) {
            Ok(_) => {
                assert!(false, "should fail to find custom 404 file")
            }
            Err(e) => {
                log_info(format!("Missing 404 file error: {:?}", e));
                assert!(true)
            }
        };
    }
}
