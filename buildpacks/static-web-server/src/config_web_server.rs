use std::fs;

use crate::{StaticWebServerBuildpack, StaticWebServerBuildpackError};
use libcnb::layer::{LayerRef};
use libcnb::{build::BuildContext, layer::UncachedLayerDefinition};
use libcnb::data::layer_name;
use libcnb::read_toml_file;
use libherokubuildpack::log::log_info;
use libherokubuildpack::toml::toml_select_value;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use toml::{toml, Table};

const DEFAULT_DOC_ROOT: &str = "public";

pub(crate) fn config_web_server(
    context: &BuildContext<StaticWebServerBuildpack>,
) -> Result<
        LayerRef<StaticWebServerBuildpack, (), ()>, 
        libcnb::Error<StaticWebServerBuildpackError>> {
    
    let configuration_layer = context.uncached_layer(
        layer_name!("configuration"),
        UncachedLayerDefinition {
            build: false,
            launch: true,
        },
    )?;

    let project_toml: toml::Value = read_toml_file(context.app_dir.join("project.toml"))
        .unwrap_or_else(|_| 
            toml::Value::try_from(toml::Table::new())
                .expect("should return the TOML Value of an empty TOML Table"));

    let doc_root = toml_select_value(
        vec!["_", "metadata", "web-server", "root"], 
        &project_toml)
        .and_then(toml::Value::as_str)
        .unwrap_or(DEFAULT_DOC_ROOT);

    // Default router is just the static file server. 
    // This vector will contain all routes in order of request processing,
    // while response processing is reverse direction.
    let mut routes: Vec<CaddyHTTPServerRoute> = vec![
        CaddyHTTPServerRoute {
            r#match: None,
            handle: vec![
                CaddyHTTPServerRouteHandler::FileServer(FileServer {
                    handler: "file_server".to_owned(),
                    root: doc_root.to_string(),
                }),
            ]
        },
    ];

    generate_response_headers_routes(project_toml, &mut routes);

    // Assemble into the caddy.json structure
    // https://caddyserver.com/docs/json/
    let caddy_config = CaddyConfig {
        apps: CaddyConfigApps { 
            http: CaddyConfigAppHTTP { 
                servers: CaddyConfigHTTPServers { 
                    public: CaddyConfigHTTPServerPublic { 
                        listen: vec![":{env.PORT}".to_owned()], 
                        routes: routes,
                    }
                }
            }
        }
    };

    let caddy_config_json = serde_json::to_string(&caddy_config)
        .map_err(StaticWebServerBuildpackError::JSON)?;

    println!("caddy.json {:?}", caddy_config_json);

    fs::write(configuration_layer.path().join("caddy.json"), caddy_config_json)
        .map_err(StaticWebServerBuildpackError::File)?;

    Ok(configuration_layer)
}

fn generate_response_headers_routes(project_toml: toml::Value, routes: &mut Vec<CaddyHTTPServerRoute>) {
    let default_toml = toml::Table::new();
    let response_headers = toml_select_value(
        vec!["_", "metadata", "web-server", "headers"], 
        &project_toml)
        .and_then(toml::Value::as_table)
        .unwrap_or(&default_toml);
    
    response_headers.iter().for_each(|(k, v)| {
        // Detect when header is defined without a path matcher (the value is not a table, probably a string)
        let headers_for_match = v.as_table().unwrap_or(&default_toml);
        // Default to * path matcher, when missing, otherwise use the current key as path matcher
        let header_match = if headers_for_match.is_empty() {
            vec![
                CaddyHTTPServerRouteMatcher::Path(MatchPath { path: vec!["*".to_string()] })
            ]
        } else {
            vec![
                CaddyHTTPServerRouteMatcher::Path(MatchPath { path: vec![k.to_string()] })
            ]
        };
        // When header is defined without path matcher, reset to configure header directly with key & value
        let header_values_to_config = if headers_for_match.is_empty() {
            let mut t = toml::Table::new();
            t.insert(k.to_string(), v.to_owned());
            t
        } else {
            headers_for_match.to_owned()
        };
        let mut header_values = Map::new();
        header_values_to_config.iter().for_each(|(kk, vv)| {
            header_values.insert(
                kk.to_string(),
                Value::Array(vec![
                    Value::String(vv.as_str().unwrap_or_default().to_string())
                ]),  
            );
        });
        let new_route = CaddyHTTPServerRoute {
            r#match: Some(header_match),
            handle: vec![
                CaddyHTTPServerRouteHandler::Headers(Headers {
                    handler: "headers".to_owned(),
                    response: HeadersResponse {
                        set: header_values,
                        deferred: true,
                    },
                }),
            ]
        };
        routes.insert(0, new_route);
    });
}

// Caddy JSON config:
// {
//     "apps": {
//         "http": {
//             "servers": {
//                 "public": {
//                     "listen": [":{env.PORT}"],
//                     "routes": [
//                         {
//                             "handle": [
//                                 {
//                                     "handler": "file_server",
//                                     "root": "public/"
//                                 }
//                             ]
//                         }
//                     ]
//                 }
//             }
//         }
//     }
// }

#[derive(Serialize, Deserialize)]
struct CaddyConfig {
    apps: CaddyConfigApps
}

#[derive(Serialize, Deserialize)]
struct CaddyConfigApps {
    http: CaddyConfigAppHTTP
}

#[derive(Serialize, Deserialize)]
struct CaddyConfigAppHTTP {
    servers: CaddyConfigHTTPServers
}

#[derive(Serialize, Deserialize)]
struct CaddyConfigHTTPServers {
    public: CaddyConfigHTTPServerPublic
}

#[derive(Serialize, Deserialize)]
struct CaddyConfigHTTPServerPublic {
    listen: Vec<String>,
    routes: Vec<CaddyHTTPServerRoute>
}

#[derive(Serialize, Deserialize)]
struct CaddyHTTPServerRoute {
    r#match: Option<Vec<CaddyHTTPServerRouteMatcher>>,
    handle: Vec<CaddyHTTPServerRouteHandler>
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum CaddyHTTPServerRouteMatcher {
    // https://caddyserver.com/docs/json/apps/http/servers/routes/match/path/
    Path(MatchPath),
}

// https://caddyserver.com/docs/json/apps/http/servers/routes/match/path/
#[derive(Serialize, Deserialize)]
struct MatchPath { 
    path: Vec<String>
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum CaddyHTTPServerRouteHandler {
    FileServer(FileServer),
    Headers(Headers)
}

// https://caddyserver.com/docs/json/apps/http/servers/routes/handle/file_server/
#[derive(Serialize, Deserialize)]
struct FileServer {
    handler: String,
    root: String,
}

// https://caddyserver.com/docs/json/apps/http/servers/routes/handle/headers/
#[derive(Serialize, Deserialize)]
struct Headers {
    handler: String,
    response: HeadersResponse,
}

#[derive(Serialize, Deserialize)]
struct HeadersResponse {
    set: Map<String, Value>,
    deferred: bool,
}

#[cfg(test)]
mod tests {
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
        generate_response_headers_routes(project_toml, &mut routes);
        assert!(routes.len() == 2,
            "should generate two routes");
        
        // First route
        let generated_route = &routes[0];
        let generated_match = generated_route.r#match.as_ref().expect("route should contain match");
        let expected_matcher = 
            if let CaddyHTTPServerRouteMatcher::Path(m) = 
                &generated_match[0] {m} else { unreachable!() };
        assert!(expected_matcher.path[0] == "*.html",
            "should have match path *.html");
        
        let generated_handler = 
            if let CaddyHTTPServerRouteHandler::Headers(h) = 
                &generated_route.handle[0] {h} else { unreachable!() };
        assert!(generated_handler.handler == "headers",
            "should be a headers route");
        
        let generated_headers_to_set = &generated_handler.response.set;
        assert!(generated_headers_to_set.contains_key("X-Baz"),
            "should contain header X-Baz");

        let expected_key = "X-Baz";
        let expected_value = Value::Array(vec![
            Value::String("Buz".to_string())
        ]);
        
        // Second route
        let generated_route = &routes[1];
        let generated_match = generated_route.r#match.as_ref().expect("route should contain match");
        let expected_matcher = 
            if let CaddyHTTPServerRouteMatcher::Path(m) = 
                &generated_match[0] {m} else { unreachable!() };
        assert!(expected_matcher.path[0] == "*",
            "should have match path *");

        let generated_handler = 
            if let CaddyHTTPServerRouteHandler::Headers(h) = 
                &generated_route.handle[0] {h} else { unreachable!() };
        assert!(generated_handler.handler == "headers",
            "should be a headers route");
        
        let generated_headers_to_set = &generated_handler.response.set;
        assert!(generated_headers_to_set.contains_key("X-Foo"),
            "should contain header X-Foo");

        let expected_key = "X-Foo";
        let expected_value = Value::Array(vec![
            Value::String("Bar".to_string())
        ]);
        assert!(generated_headers_to_set.get(expected_key) == Some(&expected_value),
            "should contain header value Bar");
        
        let expected_key = "X-Zuu";
        let expected_value = Value::Array(vec![
            Value::String("Zem".to_string())
        ]);
        assert!(generated_headers_to_set.get(expected_key) == Some(&expected_value),
            "should contain header value Zem");
    }

    #[test]
    fn generates_global_response_headers_routes() {
        let project_toml = toml::Value::Table(toml! {
            ["_".metadata.web-server.headers]
            X-Foo = "Bar"
        });
        let mut routes: Vec<CaddyHTTPServerRoute> = vec![];
        generate_response_headers_routes(project_toml, &mut routes);
        assert!(routes.len() == 1,
            "should generate one route");
        
        let generated_route = &routes[0];
        let generated_match = generated_route.r#match.as_ref().expect("route should contain match");
        let expected_matcher = 
            if let CaddyHTTPServerRouteMatcher::Path(m) = 
                &generated_match[0] {m} else { unreachable!() };
        assert!(expected_matcher.path[0] == "*",
            "should have match path *");
        
        let generated_handler = 
            if let CaddyHTTPServerRouteHandler::Headers(h) = 
                &generated_route.handle[0] {h} else { unreachable!() };
        assert!(generated_handler.handler == "headers",
            "should be a headers route");
        
        let generated_headers_to_set = &generated_handler.response.set;
        assert!(generated_headers_to_set.contains_key("X-Foo"),
            "should contain header X-Foo");

        let expected_key = "X-Foo";
        let expected_value = Value::Array(vec![
            Value::String("Bar".to_string())
        ]);
        assert!(generated_headers_to_set.get(expected_key) == Some(&expected_value),
            "should contain header value Bar");
    }
}
