mod config_web_server;
mod errors;
mod install_web_server;

use crate::errors::StaticWebServerBuildpackError;
use config_web_server::config_web_server;
use install_web_server::install_web_server;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack};
use libherokubuildpack::log::log_header;

const BUILDPACK_NAME: &str = "Heroku Static Web Server Buildpack";
const WEB_SERVER_NAME: &str = "caddy";
const WEB_SERVER_VERSION: &str = "2.8.4";

pub(crate) struct StaticWebServerBuildpack;

impl Buildpack for StaticWebServerBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = StaticWebServerBuildpackError;

    fn detect(&self, _context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        DetectResultBuilder::pass().build()
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        log_header(BUILDPACK_NAME);

        let _installation_layer =
            install_web_server(&context, WEB_SERVER_NAME, WEB_SERVER_VERSION)?;

        let configuration_layer = config_web_server(&context)?;

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
}

impl From<StaticWebServerBuildpackError> for libcnb::Error<StaticWebServerBuildpackError> {
    fn from(value: StaticWebServerBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

buildpack_main!(StaticWebServerBuildpack);
