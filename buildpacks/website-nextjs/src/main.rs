mod errors;

use crate::errors::{on_error, WebsiteNextjsBuildpackError};
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

const BUILDPACK_NAME: &str = "Heroku Website (Next.js) Buildpack";

pub(crate) struct WebsiteNextjsBuildpack;

impl Buildpack for WebsiteNextjsBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = WebsiteNextjsBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let depends_on_nextjs =
            if let Ok(package_json) = PackageJson::read(context.app_dir.join("package.json")) {
                if package_json.has_dependencies() {
                    [
                        package_json.dependencies.as_ref(),
                        package_json.dev_dependencies.as_ref(),
                    ]
                    .iter()
                    .any(|dep_group| dep_group.is_some_and(|deps| deps.contains_key("next")))
                } else {
                    false
                }
            } else {
                false
            };

        let mut static_web_server_req = Require::new("static-web-server");

        static_web_server_req
            .metadata(toml! {
                root = "/workspace/out"
                index = "index.html"

                [errors.404]
                file_path = "_not-found.html"
                status = 404
            })
            .map_err(WebsiteNextjsBuildpackError::SettingBuildPlanMetadata)?;

        let nodejs_require = Require::new("heroku/nodejs");

        let plan_builder = BuildPlanBuilder::new()
            .requires(static_web_server_req)
            .requires(nodejs_require);

        if depends_on_nextjs {
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

impl From<WebsiteNextjsBuildpackError> for libcnb::Error<WebsiteNextjsBuildpackError> {
    fn from(value: WebsiteNextjsBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

buildpack_main!(WebsiteNextjsBuildpack);
