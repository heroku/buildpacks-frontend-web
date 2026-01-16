mod errors;

use crate::errors::{on_error, WebsiteEmberBuildpackError};
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
        let contents = std::fs::read_to_string(context.app_dir.join("package.json"))
            .map_err(WebsiteEmberBuildpackError::ReadPackageJson)?;
        let json = serde_json::from_str::<serde_json::Value>(&contents)
            .map_err(WebsiteEmberBuildpackError::ParsePackageJson)?;
        let depends_on_ember_cli = json["dependencies"]
            .as_object()
            .is_some_and(|deps| deps.contains_key("ember-cli"))
            || json["devDependencies"]
                .as_object()
                .is_some_and(|deps| deps.contains_key("ember-cli"));

        let mut static_web_server_req = Require::new("static-web-server");

        static_web_server_req
            .metadata(toml! {
                root = "/workspace/dist"
                index = "index.html"

                [errors.404]
                file_path = "index.html"
                status = 200
            })
            .map_err(WebsiteEmberBuildpackError::SettingBuildPlanMetadata)?;

        let nodejs_require = Require::new("heroku/nodejs");

        let plan_builder = BuildPlanBuilder::new()
            .requires(static_web_server_req)
            .requires(nodejs_require);

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
