use std::fs;

use crate::{StaticWebServerBuildpack, StaticWebServerBuildpackError};
use libcnb::layer::{LayerRef};
use libcnb::{build::BuildContext, layer::UncachedLayerDefinition};
use libcnb::data::layer_name;
use libcnb::read_toml_file;
use libherokubuildpack::log::log_info;
use libherokubuildpack::toml::toml_select_value;
use serde::{Deserialize, Serialize};
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

    let project_toml_data = read_toml_file(context.app_dir.join("project.toml"))
        .map_err(StaticWebServerBuildpackError::ReadTOMLFile)?;
    let project_toml: toml::Value = project_toml_data;

    let doc_root = toml_select_value(
        vec!["_", "metadata", "web-server", "root"], 
        &project_toml)
        .and_then(toml::Value::as_str)
        .unwrap_or(DEFAULT_DOC_ROOT);

    let caddy_config = CaddyConfig {
        apps: CaddyConfigApps { 
            http: CaddyConfigAppHTTP { 
                servers: CaddyConfigHTTPServers { 
                    public: CaddyConfigHTTPServerPublic { 
                        listen: vec![":{env.PORT}".to_owned()], 
                        routes: vec![
                            CaddyHTTPServerRouteHandle {
                                handler: "file_server".to_owned(),
                                root: doc_root.to_string(),
                            }
                        ]
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
    routes: Vec<CaddyHTTPServerRouteHandle>
}

#[derive(Serialize, Deserialize)]
struct CaddyHTTPServerRouteHandle {
    handler: String,
    root: String
}
