mod errors;
mod o11y;

use crate::errors::{on_error, WebsiteViteBuildpackError};
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

const BUILDPACK_NAME: &str = "Heroku Website (Vite) Buildpack";

pub(crate) struct WebsiteViteBuildpack;

impl Buildpack for WebsiteViteBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = WebsiteViteBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let contents = match std::fs::read_to_string(context.app_dir.join("package.json")) {
            Ok(contents) => contents,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return DetectResultBuilder::fail().build()
            }
            Err(e) => {
                return Err(libcnb::Error::BuildpackError(
                    WebsiteViteBuildpackError::ReadPackageJson(e),
                ))
            }
        };
        let json = serde_json::from_str::<serde_json::Value>(&contents)
            .map_err(WebsiteViteBuildpackError::ParsePackageJson)?;
        let depends_on_vite = json["dependencies"]
            .as_object()
            .is_some_and(|deps| deps.contains_key("vite"))
            || json["devDependencies"]
                .as_object()
                .is_some_and(|deps| deps.contains_key("vite"));

        let mut static_web_server_req = Require::new("static-web-server");

        static_web_server_req
            .metadata(toml! {
                root = "/workspace/dist"
                index = "index.html"

                [errors.404]
                file_path = "index.html"
                status = 200
            })
            .map_err(WebsiteViteBuildpackError::SettingBuildPlanMetadata)?;

        let nodejs_require = Require::new("heroku/nodejs");

        let plan_builder = BuildPlanBuilder::new()
            .requires(static_web_server_req)
            .requires(nodejs_require);

        if depends_on_vite {
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

impl From<WebsiteViteBuildpackError> for libcnb::Error<WebsiteViteBuildpackError> {
    fn from(value: WebsiteViteBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

buildpack_main!(WebsiteViteBuildpack);
