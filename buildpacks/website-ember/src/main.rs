mod errors;

use crate::errors::{on_error, WebsiteEmberBuildpackError};
use heroku_nodejs_utils::package_json::PackageJson;
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

const BUILDPACK_NAME: &str = "Heroku Website (Ember.js) Buildpack";

pub(crate) struct WebsiteEmberBuildpack;

impl Buildpack for WebsiteEmberBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = WebsiteEmberBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let depends_on_ember_cli =
            if let Ok(package_json) = PackageJson::read(context.app_dir.join("package.json")) {
                if package_json.has_dependencies() {
                    [
                        package_json.dependencies.as_ref(),
                        package_json.dev_dependencies.as_ref(),
                    ]
                    .iter()
                    .any(|dep_group| dep_group.is_some_and(|deps| deps.contains_key("ember-cli")))
                } else {
                    false
                }
            } else {
                false
            };

        let mut static_web_server_req = Require::new("static-web-server");
        static_web_server_req
            .metadata(toml! {
                // The package.json build script will automatically execute by heroku/nodejs buildpack.
                // Eventually, that build execution will be made optional, so this one will take over.
                // build.command = "sh"
                // build.args = ["-c", "ember build --environment=production"]

                root = "/workspace/static-artifacts"
                index = "index.html"

                [errors.404]
                file_path = "index.html"
                status = 200
            })
            .map_err(WebsiteEmberBuildpackError::SettingBuildPlanMetadata)?;

        let mut release_phase_req = Require::new("release-phase");

        let mut release_phase_metadata = toml::Table::new();
        let mut release_build_command = toml::Table::new();
        release_build_command.insert("command".to_string(), "bash".to_string().into());
        release_build_command.insert(
            "args".to_string(),
            vec![
                "-c",
                "npm run build && mkdir -p static-artifacts && cp -rL dist/* static-artifacts/",
            ]
            .into(),
        );
        release_build_command.insert("source".to_string(), BUILDPACK_NAME.to_string().into());

        release_phase_metadata.insert("release-build".to_string(), release_build_command.into());
        release_phase_req
            .metadata(release_phase_metadata)
            .map_err(WebsiteEmberBuildpackError::SettingBuildPlanMetadata)?;

        let plan_builder = BuildPlanBuilder::new()
            .requires(static_web_server_req)
            .requires(release_phase_req);

        if depends_on_ember_cli {
            DetectResultBuilder::pass()
                .build_plan(plan_builder.build())
                .build()
        } else {
            DetectResultBuilder::fail().build()
        }
    }

    fn build(&self, _context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        log_header(BUILDPACK_NAME);
        BuildResultBuilder::new().build()
    }

    fn on_error(&self, error: Error<Self::Error>) {
        on_error(error);
    }
}

impl From<WebsiteEmberBuildpackError> for libcnb::Error<WebsiteEmberBuildpackError> {
    fn from(value: WebsiteEmberBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

buildpack_main!(WebsiteEmberBuildpack);
