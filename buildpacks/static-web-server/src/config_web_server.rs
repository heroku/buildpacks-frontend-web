use std::fs;

use crate::{StaticWebServerBuildpack, StaticWebServerBuildpackError};
use libcnb::layer::{LayerRef};
use libcnb::{build::BuildContext, layer::UncachedLayerDefinition};
use libcnb::data::layer_name;

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

    let default_caddy_config = r#"
    {
        "apps": {
            "http": {
                "servers": {
                    "public": {
                        "listen": [":{env.PORT}"],
                        "routes": [
                            {
                                "handle": [
                                    {
                                        "handler": "file_server",
                                        "root": "public/"
                                    }
                                ]
                            }
                        ]
                    }
                }
            }
        }
    }
    "#;

    fs::write(configuration_layer.path().join("caddy.json"), default_caddy_config)
        .map_err(StaticWebServerBuildpackError::File)?;

    Ok(configuration_layer)
}
