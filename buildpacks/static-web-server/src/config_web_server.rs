use itertools::Itertools;
use std::fs;
use std::path::PathBuf;

use crate::caddy_config::*;
use crate::errors::StaticWebServerBuildpackError::CannotReadCustom404File;
use crate::heroku_web_server_config::{Header, HeaderPathMatcher, HerokuWebServerConfig};
use crate::{StaticWebServerBuildpack, StaticWebServerBuildpackError};
use libcnb::data::layer_name;
use libcnb::layer::LayerRef;
use libcnb::read_toml_file;
use libcnb::{build::BuildContext, layer::UncachedLayerDefinition};
use libherokubuildpack::log::log_info;
use libherokubuildpack::toml::toml_select_value;

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

    let project_toml_path = context.app_dir.join("project.toml");

    let heroku_config = if project_toml_path.is_file() {
        let project_toml = read_toml_file::<toml::Value>(project_toml_path)
            .map_err(StaticWebServerBuildpackError::CannotReadProjectToml)?;

        toml_select_value(vec!["_", "metadata", "web-server"], &project_toml)
            .unwrap()
            .clone()
            .try_into()
            .unwrap()
    } else {
        HerokuWebServerConfig::default()
    };

    let mut routes = vec![];

    // Header routes come first so headers will be added to any response down the chain.
    routes.extend(generate_response_headers_routes(&heroku_config.headers));

    // Default router is just the static file server.
    // This vector will contain all routes in order of request processing,
    // while response processing is reverse direction.
    routes.push(CaddyHTTPServerRoute {
        r#match: None,
        handle: vec![CaddyHTTPServerRouteHandler::FileServer(FileServer {
            handler: "file_server".to_owned(),
            root: heroku_config
                .root
                .clone()
                .unwrap_or(PathBuf::from(DEFAULT_DOC_ROOT))
                .to_string_lossy()
                .to_string(),
            // Any not found request paths continue to the next handler.
            pass_thru: true,
        })],
    });

    routes.push(generate_error_404_route(&heroku_config, &context.app_dir)?);

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
    fs::write(&config_path, caddy_config_json)
        .map_err(StaticWebServerBuildpackError::CannotWriteCaddyConfiguration)?;

    Ok(configuration_layer)
}

fn generate_response_headers_routes(headers: &[Header]) -> Vec<CaddyHTTPServerRoute> {
    headers
        .into_iter()
        .chunk_by(|header| header.path_matcher.clone())
        .into_iter()
        .map(|(matcher, headers)| {
            let match_path = match matcher {
                HeaderPathMatcher::Global => String::from("*"),
                HeaderPathMatcher::Path(path) => path,
            };

            CaddyHTTPServerRoute {
                r#match: Some(vec![CaddyHTTPServerRouteMatcher::Path(MatchPath {
                    path: vec![match_path],
                })]),
                handle: vec![CaddyHTTPServerRouteHandler::Headers(Headers {
                    handler: "headers".to_owned(),
                    response: HeadersResponse {
                        set: headers
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
            }
        })
        .collect()
}

fn generate_error_404_route(
    heroku_web_server_config: &HerokuWebServerConfig,
    app_dir: &PathBuf,
) -> Result<CaddyHTTPServerRoute, StaticWebServerBuildpackError> {
    let not_found_response_content = heroku_web_server_config
        .errors
        .as_ref()
        .and_then(|errors| errors.custom_404_page.clone())
        .map(|path| fs::read_to_string(app_dir.join(path)).map_err(CannotReadCustom404File))
        .unwrap_or({
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
        })?;

    Ok(CaddyHTTPServerRoute {
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
                body: not_found_response_content,
            },
        )],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, path::PathBuf};
    use toml::toml;

    #[test]
    fn generates_matched_response_headers_routes() {
        let project_toml = toml::Value::Table(toml! {
            [headers]
            "*".X-Foo = "Bar"
            "*.html".X-Baz = "Buz"
            "*".X-Zuu = "Zem"
        });

        let heroku_config = project_toml.try_into::<HerokuWebServerConfig>().unwrap();

        let routes = generate_response_headers_routes(&heroku_config.headers);
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
            [headers]
            X-Foo = "Bar"
        });

        let heroku_config = project_toml.try_into::<HerokuWebServerConfig>().unwrap();
        let routes = generate_response_headers_routes(&heroku_config.headers);
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
            [errors]
            404 = "public/error-404.html"
        });

        let heroku_config = project_toml.try_into::<HerokuWebServerConfig>().unwrap();

        let app_dir =
            PathBuf::from(env::current_dir().unwrap()).join("tests/fixtures/custom_errors");

        let routes = generate_error_404_route(&heroku_config, &app_dir).unwrap();

        let generated_handler =
            if let CaddyHTTPServerRouteHandler::StaticResponse(h) = &routes.handle[0] {
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
            [errors]
            404 = "non-existent-path"
        });

        let heroku_config = project_toml.try_into::<HerokuWebServerConfig>().unwrap();

        let app_dir =
            PathBuf::from(env::current_dir().unwrap()).join("tests/fixtures/custom_errors");

        match generate_error_404_route(&heroku_config, &app_dir) {
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
