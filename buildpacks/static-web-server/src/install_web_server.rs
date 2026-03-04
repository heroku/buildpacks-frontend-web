use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, EmptyLayerCause, InvalidMetadataAction, LayerRef, LayerState,
    RestoredLayerAction,
};
use libherokubuildpack::download::download_file;
use libherokubuildpack::log::log_info;
use libherokubuildpack::tar::decompress_tarball;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use tempfile::NamedTempFile;

use crate::o11y::*;
use crate::{StaticWebServerBuildpack, StaticWebServerBuildpackError};

pub(crate) fn install_web_server(
    context: &BuildContext<StaticWebServerBuildpack>,
    web_server_name: &str,
    web_server_version: &str,
) -> Result<
    LayerRef<StaticWebServerBuildpack, (), Vec<std::string::String>>,
    libcnb::Error<StaticWebServerBuildpackError>,
> {
    let new_metadata = WebServerLayerMetadata {
        web_server_name: web_server_name.to_string(),
        web_server_version: web_server_version.to_string(),
        arch: context.target.arch.clone(),
        os: context.target.os.clone(),
    };

    let installation_layer = context.cached_layer(
        layer_name!("installation"),
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
    match installation_layer.state {
        LayerState::Restored { .. } => {
            log_info("Using cached web server");
        }
        LayerState::Empty { ref cause } => {
            installation_layer.write_metadata(new_metadata)?;

            if let EmptyLayerCause::RestoredLayerAction { cause } = cause {
                log_info(format!(
                    "Invalidating cached web server ({} changed)",
                    cause.join(", ")
                ));
            }

            let artifact_url = format!(
                "https://github.com/caddyserver/caddy/releases/download/v{}/caddy_{}_{}_{}.tar.gz",
                web_server_version, web_server_version, context.target.os, context.target.arch
            );

            let web_server_tgz = NamedTempFile::new()
                .map_err(StaticWebServerBuildpackError::CannotCreateCaddyTarballFile)?;
            let web_server_dir = installation_layer.path().join("bin");
            fs::create_dir_all(&web_server_dir)
                .map_err(StaticWebServerBuildpackError::CannotCreateCaddyInstallationDir)?;

            log_info(format!("Downloading web server from {artifact_url}"));
            tracing::info!(
                { INSTALLATION_WEB_SERVER_NAME } = web_server_name,
                { INSTALLATION_WEB_SERVER_VERSION } = web_server_version,
                "downloading web server"
            );
            download_file(&artifact_url, web_server_tgz.path())
                .map_err(StaticWebServerBuildpackError::Download)?;

            // Verify the checksum
            log_info("Verifying web server checksum");
            verify_caddy_checksum(
                web_server_version,
                &context.target.os,
                &context.target.arch,
                web_server_tgz.path(),
            )?;

            decompress_tarball(&mut web_server_tgz.into_file(), &web_server_dir)
                .map_err(StaticWebServerBuildpackError::CannotUnpackCaddyTarball)?;
        }
    }
    Ok(installation_layer)
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

/// Verifies the Caddy binary checksum against the official checksums file
fn verify_caddy_checksum(
    version: &str,
    os: &str,
    arch: &str,
    tarball_path: &Path,
) -> Result<(), libcnb::Error<StaticWebServerBuildpackError>> {
    let base_url = format!("https://github.com/caddyserver/caddy/releases/download/v{version}");
    let checksums_filename = format!("caddy_{version}_checksums.txt");
    let artifact_filename = format!("caddy_{version}_{os}_{arch}.tar.gz");

    // Download checksums file
    let checksums_file = NamedTempFile::new()
        .map_err(StaticWebServerBuildpackError::CannotCreateCaddyTarballFile)?;
    download_file(
        format!("{base_url}/{checksums_filename}"),
        checksums_file.path(),
    )
    .map_err(StaticWebServerBuildpackError::Download)?;

    // Verify the tarball checksum against the checksums file
    verify_checksum(tarball_path, checksums_file.path(), &artifact_filename)?;

    tracing::info!("Successfully verified Caddy checksum for version {version}");

    Ok(())
}

/// Verifies the checksum of a file against a checksums file
fn verify_checksum(
    file_path: &Path,
    checksums_path: &Path,
    expected_filename: &str,
) -> Result<(), libcnb::Error<StaticWebServerBuildpackError>> {
    // Calculate the SHA512 hash of the downloaded file
    let mut file =
        fs::File::open(file_path).map_err(StaticWebServerBuildpackError::CannotReadChecksums)?;
    let mut hasher = Sha512::new();
    std::io::copy(&mut file, &mut hasher)
        .map_err(StaticWebServerBuildpackError::CannotReadChecksums)?;
    let calculated_hash = format!("{:x}", hasher.finalize());

    // Parse the checksums file to find the expected checksum
    let checksums_file = fs::File::open(checksums_path)
        .map_err(StaticWebServerBuildpackError::CannotReadChecksums)?;
    let reader = BufReader::new(checksums_file);

    let mut found_checksum = None;
    for line in reader.lines() {
        let line = line.map_err(StaticWebServerBuildpackError::CannotReadChecksums)?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == expected_filename {
            found_checksum = Some(parts[0].to_string());
            break;
        }
    }

    let expected_hash = found_checksum.ok_or_else(|| {
        StaticWebServerBuildpackError::ChecksumVerificationFailed(format!(
            "Checksum for {expected_filename} not found in checksums file"
        ))
    })?;

    // Compare checksums
    if calculated_hash != expected_hash {
        return Err(
            StaticWebServerBuildpackError::ChecksumVerificationFailed(format!(
                "Checksum mismatch for {expected_filename}: expected {expected_hash}, got {calculated_hash}"
            ))
            .into(),
        );
    }

    Ok(())
}
