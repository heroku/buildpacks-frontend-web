mod errors;
mod o11y;

use std::process::Command;

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

    fn build(&self, _context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        log_header(BUILDPACK_NAME);

        // Ensure comptibility with the static web server,
        // that Next's build output is set to static exports.
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "npm exec next info"]);
        let next_info = cmd
            .output()
            .map_err(WebsiteNextjsBuildpackError::NextInfoCommandError)?;
        let next_info_stdout = String::from_utf8_lossy(&next_info.stdout);
        let next_info_stderr = String::from_utf8_lossy(&next_info.stderr);
        if !next_info.status.success() {
            return Err(libcnb::Error::BuildpackError(
                WebsiteNextjsBuildpackError::NextInfoFailure(next_info_stderr.to_string()),
            ));
        }
        if !next_info_stdout.contains(&"output: export".to_string()) {
            return Err(libcnb::Error::BuildpackError(
                WebsiteNextjsBuildpackError::RequiresStaticExport,
            ));
        }

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
