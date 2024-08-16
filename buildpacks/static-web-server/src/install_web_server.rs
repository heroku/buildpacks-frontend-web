use std::fs;

use libcnb::{build::BuildContext, layer::UncachedLayerDefinition};
use libcnb::data::layer_name;
use libherokubuildpack::download::download_file;
use libherokubuildpack::fs::move_directory_contents;
use libherokubuildpack::log::log_info;
use libherokubuildpack::tar::decompress_tarball;
use tempfile::NamedTempFile;

use crate::{StaticWebServerBuildpack, StaticWebServerBuildpackError, WEB_SERVER_BIN_DIR};

pub(crate) fn install_web_server(
    context: &BuildContext<StaticWebServerBuildpack>,
) -> Result<(), libcnb::Error<StaticWebServerBuildpackError>> {
    
    let layer_ref = context.uncached_layer(
        layer_name!("web_server"),
        UncachedLayerDefinition {
            build: true,
            launch: true,
        },
    )?;

    let artifact_url = format!(
        "https://github.com/caddyserver/caddy/releases/download/v2.8.4/caddy_2.8.4_{}_{}.tar.gz", 
        context.target.os, context.target.arch
    );

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

    let web_server_tgz = NamedTempFile::new().unwrap();
    let web_server_dir = layer_ref.path()
        .join(WEB_SERVER_BIN_DIR);
    fs::create_dir_all(&web_server_dir).unwrap();
    
    log_info(format!(
        "Downloading web server from {}",
        artifact_url
    ));
    download_file(artifact_url, web_server_tgz.path())
        .map_err(StaticWebServerBuildpackError::Download)?;

    decompress_tarball(&mut web_server_tgz.into_file(), &web_server_dir).unwrap();

    fs::write(layer_ref.path().join("caddy.json"), default_caddy_config).unwrap();

    Ok(())
}
