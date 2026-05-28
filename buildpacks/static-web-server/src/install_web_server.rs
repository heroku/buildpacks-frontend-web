use std::fs;
use std::path::Path;

use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, EmptyLayerCause, InvalidMetadataAction, LayerRef, LayerState,
    RestoredLayerAction,
};
use libherokubuildpack::download::download_file;
use libherokubuildpack::inventory::artifact::Artifact;
use libherokubuildpack::inventory::checksum::Checksum;
use libherokubuildpack::log::log_info;
use libherokubuildpack::tar::decompress_tarball;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

use crate::o11y::*;
use crate::{StaticWebServerBuildpack, StaticWebServerBuildpackError};

type WebServerArtifact = Artifact<Version, Sha256, Option<()>>;

pub(crate) fn install_web_server(
    context: &BuildContext<StaticWebServerBuildpack>,
    artifact: &WebServerArtifact,
) -> Result<
    LayerRef<StaticWebServerBuildpack, (), Option<WebServerArtifact>>,
    libcnb::Error<StaticWebServerBuildpackError>,
> {
    let new_metadata = WebServerLayerMetadata {
        artifact: artifact.clone(),
    };

    let installation_layer = context.cached_layer(
        layer_name!("installation"),
        CachedLayerDefinition {
            build: false,
            launch: true,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &WebServerLayerMetadata, _| {
                if old_metadata == &new_metadata {
                    (RestoredLayerAction::KeepLayer, None)
                } else {
                    (
                        RestoredLayerAction::DeleteLayer,
                        Some(old_metadata.artifact.clone()),
                    )
                }
            },
        },
    )?;
    match installation_layer.state {
        LayerState::Restored { .. } => {
            log_info("Using cached web server");
        }
        LayerState::Empty { ref cause } => {
            if let EmptyLayerCause::RestoredLayerAction {
                cause: Some(old_artifact),
            } = cause
            {
                log_info(format!(
                    "Invalidating cached web server (Caddy {})",
                    old_artifact.version
                ));
            }
            installation_layer.write_metadata(new_metadata)?;

            let web_server_tgz = NamedTempFile::new()
                .map_err(StaticWebServerBuildpackError::CannotCreateCaddyTarballFile)?;
            let web_server_dir = installation_layer.path().join("bin");
            fs::create_dir_all(&web_server_dir)
                .map_err(StaticWebServerBuildpackError::CannotCreateCaddyInstallationDir)?;

            log_info(format!("Downloading web server from {}", artifact.url));
            tracing::info!(
                { INSTALLATION_WEB_SERVER_NAME } = crate::WEB_SERVER_NAME,
                { INSTALLATION_WEB_SERVER_VERSION } = %artifact.version,
                "downloading web server"
            );
            download_file(&artifact.url, web_server_tgz.path())
                .map_err(StaticWebServerBuildpackError::Download)?;

            log_info("Verifying web server checksum");
            verify_checksum(&artifact.checksum, web_server_tgz.path())?;

            decompress_tarball(&mut web_server_tgz.into_file(), &web_server_dir)
                .map_err(StaticWebServerBuildpackError::CannotUnpackCaddyTarball)?;
        }
    }
    Ok(installation_layer)
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct WebServerLayerMetadata {
    artifact: WebServerArtifact,
}

fn verify_checksum(
    expected: &Checksum<Sha256>,
    path: &Path,
) -> Result<(), StaticWebServerBuildpackError> {
    let actual = fs::read(path)
        .map(|bytes| Sha256::digest(&bytes).to_vec())
        .map_err(StaticWebServerBuildpackError::ReadDownloadForChecksum)?;

    if actual == expected.value {
        Ok(())
    } else {
        Err(StaticWebServerBuildpackError::ChecksumVerificationFailed {
            expected: expected.value.clone(),
            actual,
        })
    }
}
