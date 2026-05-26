mod caddy_config;
mod config_web_server;
mod errors;
mod heroku_web_server_config;
mod install_web_server;
mod o11y;

use crate::errors::{on_error, StaticWebServerBuildpackError};
use crate::o11y::*;
use config_web_server::config_web_server;
use install_web_server::install_web_server;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::{BuildPlanBuilder, Require};
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack, Error, Target};
use libherokubuildpack::inventory::artifact::{Arch, Artifact, Os};
use libherokubuildpack::inventory::Inventory;
use libherokubuildpack::log::log_header;
use semver::{Version, VersionReq};
use sha2::Sha256;

use env_as_html_data as _;
use regex as _;

// Silence unused dependency warning for
// dependencies only used in tests
#[cfg(test)]
use libcnb_test as _;
#[cfg(test)]
use test_support as _;
use ureq as _;

const BUILDPACK_NAME: &str = "Heroku Static Web Server Buildpack";
const BUILD_PLAN_ID: &str = "static-web-server";
pub(crate) const WEB_SERVER_NAME: &str = "caddy";
pub(crate) const WEB_SERVER_VERSION: &str = "2.11.2";

pub(crate) struct StaticWebServerBuildpack;

impl Buildpack for StaticWebServerBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = StaticWebServerBuildpackError;

    fn detect(&self, _context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let plan_builder = BuildPlanBuilder::new()
            .provides(BUILD_PLAN_ID)
            .requires(Require::new(BUILD_PLAN_ID));

        tracing::info!({ DETECT_PROVIDES_STATIC_WEB_SERVER } = true, "buildplan");
        tracing::info!({ DETECT_REQUIRES_STATIC_WEB_SERVER } = true, "buildplan");

        DetectResultBuilder::pass()
            .build_plan(plan_builder.build())
            .build()
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        log_header(BUILDPACK_NAME);

        let artifact = resolve_caddy_artifact(&context.target);

        let _installation_layer = install_web_server(&context, &artifact)?;

        let configuration_layer = config_web_server(&context)?;

        tracing::info!(
            { INSTALLATION_WEB_SERVER_NAME } = WEB_SERVER_NAME,
            { INSTALLATION_WEB_SERVER_VERSION } = %artifact.version,
            "build success"
        );

        BuildResultBuilder::new()
            .launch(
                LaunchBuilder::new()
                    .process(
                        ProcessBuilder::new(
                            process_type!("web"),
                            [
                                "caddy",
                                "run",
                                "--config",
                                &configuration_layer
                                    .path()
                                    .join("caddy.json")
                                    .to_string_lossy(),
                            ],
                        )
                        .default(true)
                        .build(),
                    )
                    .build(),
            )
            .build()
    }

    fn on_error(&self, error: Error<Self::Error>) {
        on_error(error);
    }
}

impl From<StaticWebServerBuildpackError> for libcnb::Error<StaticWebServerBuildpackError> {
    fn from(value: StaticWebServerBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

// Resolves the Caddy artifact pinned in the bundled inventory.toml. Both
// the inventory parse and the artifact lookup are guaranteed to succeed
// for any combination of WEB_SERVER_VERSION + buildpack-supported target
// that has shipped through CI; failures here mean the buildpack itself
// is misconfigured.
fn resolve_caddy_artifact(target: &Target) -> Artifact<Version, Sha256, Option<()>> {
    let version_req = VersionReq::parse(&format!("={WEB_SERVER_VERSION}"))
        .expect("the pinned WEB_SERVER_VERSION should always parse as an exact-match requirement");
    let inventory: Inventory<_, _, _> = include_str!("../inventory.toml")
        .parse()
        .expect("the bundled Caddy inventory should always parse");
    inventory
        .resolve(
            target.os.parse::<Os>().expect("OS should always be parseable, buildpack will not run on unsupported operating systems."),
            target.arch.parse::<Arch>().expect("Arch should always be parseable, buildpack will not run on unsupported architectures."),
            &version_req,
        )
        .cloned()
        .unwrap_or_else(|| {
            panic!(
                "the bundled Caddy inventory should always have an entry matching os={}, arch={}, version requirement {version_req}",
                target.os, target.arch,
            )
        })
}

buildpack_main!(StaticWebServerBuildpack);
