mod errors;
mod o11y;

use std::path::Path;

use crate::errors::{on_error, WebsiteNextjsBuildpackError};
use crate::o11y::*;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::{BuildPlanBuilder, Require};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack, Error};
use libherokubuildpack::log::log_header;
use toml::toml;

// Silence unused dependency warning for
// dependencies only used in tests
#[cfg(test)]
use libcnb_test as _;
#[cfg(test)]
use tempfile as _;
#[cfg(test)]
use test_support as _;
#[cfg(test)]
use ureq as _;
#[cfg(test)]
use uuid as _;

const BUILDPACK_NAME: &str = "Heroku Website (Next.js) Buildpack";

pub(crate) struct WebsiteNextjsBuildpack;

impl Buildpack for WebsiteNextjsBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = WebsiteNextjsBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let contents = match std::fs::read_to_string(context.app_dir.join("package.json")) {
            Ok(contents) => contents,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return DetectResultBuilder::fail().build()
            }
            Err(e) => {
                return Err(libcnb::Error::BuildpackError(
                    WebsiteNextjsBuildpackError::ReadPackageJson(e),
                ))
            }
        };
        let json = serde_json::from_str::<serde_json::Value>(&contents)
            .map_err(WebsiteNextjsBuildpackError::ParsePackageJson)?;
        let depends_on_nextjs = json["dependencies"]
            .as_object()
            .is_some_and(|deps| deps.contains_key("next"))
            || json["devDependencies"]
                .as_object()
                .is_some_and(|deps| deps.contains_key("next"));

        let mut static_web_server_req = Require::new("static-web-server");

        // Following Next.js static deployment guidance
        // https://nextjs.org/docs/app/guides/static-exports
        static_web_server_req
            .metadata(toml! {
                root = "/workspace/out"
                index = "index.html"

                [runtime_config]
                html_files = ["**/*.html"]

                [errors.404]
                file_path = "404.html"
                path_exclusions = ["/_next/*", "/_not-found/*", "/nested/*"]

                [caddy_server_opts]
                clean_urls = true
            })
            .map_err(WebsiteNextjsBuildpackError::SettingBuildPlanMetadata)?;

        let nodejs_require = Require::new("heroku/nodejs");

        let plan_builder = BuildPlanBuilder::new()
            .requires(static_web_server_req)
            .requires(nodejs_require);

        tracing::info!({ DETECT_PROVIDES_WEBSITE_NEXTJS } = true, "buildplan");
        tracing::info!(
            { DETECT_REQUIRES_WEBSITE_NEXTJS } = depends_on_nextjs,
            "buildplan"
        );

        if depends_on_nextjs {
            DetectResultBuilder::pass()
                .build_plan(plan_builder.build())
                .build()
        } else {
            DetectResultBuilder::fail().build()
        }
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        log_header(BUILDPACK_NAME);

        validate_static_export(&context.app_dir)?;

        BuildResultBuilder::new().build()
    }

    fn on_error(&self, error: Error<Self::Error>) {
        on_error(error);
    }
}

fn validate_static_export(app_dir: &Path) -> Result<(), WebsiteNextjsBuildpackError> {
    let required_server_files_path = app_dir.join(".next/required-server-files.json");
    let required_server_files_contents = std::fs::read_to_string(required_server_files_path)
        .map_err(WebsiteNextjsBuildpackError::ReadBuildOutputMetadata)?;
    let required_server_files: serde_json::Value =
        serde_json::from_str(&required_server_files_contents)
            .map_err(WebsiteNextjsBuildpackError::ParseBuildOutputMetadata)?;

    if required_server_files
        .pointer("/config/output")
        .and_then(serde_json::Value::as_str)
        != Some("export")
    {
        return Err(WebsiteNextjsBuildpackError::RequiresStaticExport);
    }

    let export_detail_path = app_dir.join(".next/export-detail.json");
    let export_detail_contents = std::fs::read_to_string(export_detail_path)
        .map_err(WebsiteNextjsBuildpackError::ReadBuildOutputMetadata)?;
    let export_detail: serde_json::Value = serde_json::from_str(&export_detail_contents)
        .map_err(WebsiteNextjsBuildpackError::ParseBuildOutputMetadata)?;

    if export_detail
        .get("success")
        .and_then(serde_json::Value::as_bool)
        != Some(true)
    {
        return Err(WebsiteNextjsBuildpackError::StaticExportFailed);
    }

    Ok(())
}

impl From<WebsiteNextjsBuildpackError> for libcnb::Error<WebsiteNextjsBuildpackError> {
    fn from(value: WebsiteNextjsBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

buildpack_main!(WebsiteNextjsBuildpack);

#[cfg(test)]
mod tests {
    use super::validate_static_export;
    use std::fs;
    use std::path::Path;

    fn write_build_metadata(app_dir: &Path, output: &str, success: bool) {
        let next_dir = app_dir.join(".next");
        fs::create_dir_all(&next_dir).unwrap();
        fs::write(
            next_dir.join("required-server-files.json"),
            format!(r#"{{"config":{{"output":"{output}"}}}}"#),
        )
        .unwrap();
        fs::write(
            next_dir.join("export-detail.json"),
            format!(r#"{{"success":{success}}}"#),
        )
        .unwrap();
    }

    #[test]
    fn accepts_successful_static_export() {
        let app_dir = tempfile::tempdir().unwrap();
        write_build_metadata(app_dir.path(), "export", true);

        assert!(validate_static_export(app_dir.path()).is_ok());
    }

    #[test]
    fn rejects_build_without_static_export_configuration() {
        let app_dir = tempfile::tempdir().unwrap();
        write_build_metadata(app_dir.path(), "", true);

        assert!(validate_static_export(app_dir.path()).is_err());
    }

    #[test]
    fn rejects_unsuccessful_static_export() {
        let app_dir = tempfile::tempdir().unwrap();
        write_build_metadata(app_dir.path(), "export", false);

        assert!(validate_static_export(app_dir.path()).is_err());
    }

    #[test]
    fn rejects_missing_export_metadata() {
        let app_dir = tempfile::tempdir().unwrap();
        let next_dir = app_dir.path().join(".next");
        std::fs::create_dir(&next_dir).unwrap();
        std::fs::write(
            next_dir.join("required-server-files.json"),
            r#"{"config":{"output":"export"}}"#,
        )
        .unwrap();

        assert!(validate_static_export(app_dir.path()).is_err());
    }

    #[test]
    fn rejects_malformed_build_metadata() {
        let app_dir = tempfile::tempdir().unwrap();
        let next_dir = app_dir.path().join(".next");
        std::fs::create_dir(&next_dir).unwrap();
        std::fs::write(next_dir.join("required-server-files.json"), "invalid json").unwrap();

        assert!(validate_static_export(app_dir.path()).is_err());
    }
}
