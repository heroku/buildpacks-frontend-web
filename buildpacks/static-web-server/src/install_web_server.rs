use std::fs;

use libcnb::layer::{
    CachedLayerDefinition, InvalidMetadataAction, LayerRef,LayerState, RestoredLayerAction,
};
use libcnb::{build::BuildContext, layer::UncachedLayerDefinition};
use libcnb::data::layer_name;
use libherokubuildpack::download::download_file;
use libherokubuildpack::log::log_info;
use libherokubuildpack::tar::decompress_tarball;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use crate::{StaticWebServerBuildpack, StaticWebServerBuildpackError};

pub(crate) fn install_web_server(
    context: &BuildContext<StaticWebServerBuildpack>,
    web_server_name: &str,
    web_server_version: &str,
) -> Result<
        LayerRef<StaticWebServerBuildpack, (), Vec<std::string::String>>, 
        libcnb::Error<StaticWebServerBuildpackError>> {
    
    let new_metadata = WebServerLayerMetadata {
        web_server_name: web_server_name.to_string(),
        web_server_version: web_server_version.to_string(),
        arch: context.target.arch.clone(),
        os: context.target.os.clone(),
    };

    let layer_ref = context.cached_layer(
        layer_name!("web-server"),
        CachedLayerDefinition {
            build: false,
            launch: true,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &WebServerLayerMetadata, _| {
                if old_metadata == &new_metadata {
                    Ok((RestoredLayerAction::KeepLayer, vec![]))
                } else {
                    Ok((
                        RestoredLayerAction::DeleteLayer,
                        changed_metadata_fields(old_metadata, &new_metadata),
                    ))
                }
            },
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

    let web_server_tgz = NamedTempFile::new()
        .map_err(StaticWebServerBuildpackError::File)?;
    let web_server_dir = layer_ref.path().join("bin");
    fs::create_dir_all(&web_server_dir)
        .map_err(StaticWebServerBuildpackError::File)?;
    
    log_info(format!(
        "Downloading web server from {}",
        artifact_url
    ));
    download_file(artifact_url, web_server_tgz.path())
        .map_err(StaticWebServerBuildpackError::Download)?;
    decompress_tarball(&mut web_server_tgz.into_file(), &web_server_dir)
        .map_err(StaticWebServerBuildpackError::File)?;
    
    fs::write(layer_ref.path().join("caddy.json"), default_caddy_config)
        .map_err(StaticWebServerBuildpackError::File)?;

    Ok(layer_ref)
}

fn changed_metadata_fields(
    old: &WebServerLayerMetadata,
    new: &WebServerLayerMetadata,
) -> Vec<String> {
    let mut changed = vec![];
    if old.web_server_name != new.web_server_name {
        changed.push("web server name".to_string());
    }
    if old.web_server_version != new.web_server_version {
        changed.push("web server version".to_string());
    }
    if old.os != new.os {
        changed.push("operating system".to_string());
    }
    if old.arch != new.arch {
        changed.push("compute architecture".to_string());
    }
    changed.sort();
    changed
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct WebServerLayerMetadata {
    web_server_name: String,
    web_server_version: String,
    arch: String,
    os: String,
}
