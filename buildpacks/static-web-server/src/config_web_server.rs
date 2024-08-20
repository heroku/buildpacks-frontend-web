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
use toml::toml;

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
                CaddyHTTPServerRouteHandler::FileServer {
                    handler: "file_server".to_owned(),
                    root: doc_root.to_string(),
                },
            ]
        },
    ];

    let default_toml = toml::Table::new();

    // Get configured response headers.
    let response_headers = toml_select_value(
        vec!["_", "metadata", "web-server", "headers"], 
        &project_toml)
        .and_then(toml::Value::as_table)
        .unwrap_or(&default_toml);
    // Get configured response headers.
    response_headers.iter().for_each(|(k, v)| {
        let mut kv = Map::new();
        kv.insert(
            k.to_string(),
            Value::Array(vec![
                Value::String(v.as_str().unwrap_or_default().to_string())
            ]),  
        );
        let new_route = CaddyHTTPServerRoute {
            r#match: None,
            handle: vec![
                CaddyHTTPServerRouteHandler::Headers {
                    handler: "headers".to_owned(),
                    response: HeadersResponse {
                        set: kv,
                        deferred: true,
                    },
                },
            ]
        };
        routes.insert(0, new_route);
    });

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
    fs::write(configuration_layer.path().join("caddy.json"), caddy_config_json)
        .map_err(StaticWebServerBuildpackError::File)?;

    Ok(configuration_layer)
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
    Path { 
        path: String
    },
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum CaddyHTTPServerRouteHandler {
    // https://caddyserver.com/docs/json/apps/http/servers/routes/handle/file_server/
    FileServer {
        handler: String,
        root: String,
    },
    // https://caddyserver.com/docs/json/apps/http/servers/routes/handle/headers/
    Headers {
        handler: String,
        response: HeadersResponse,
    }
}

#[derive(Serialize, Deserialize)]
struct HeadersResponse {
    set: Map<String, Value>,
    deferred: bool
}
