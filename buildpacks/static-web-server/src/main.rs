mod errors;
mod install_web_server;

use crate::errors::StaticWebServerBuildpackError;
use install_web_server::install_web_server;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack};
use libherokubuildpack::log::log_header;

const BUILDPACK_NAME: &str = "Heroku Static Web Server Buildpack";
const WEB_SERVER_BIN_DIR: &str = "bin";

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

        let web_server_layer = install_web_server(&context)?;
        let config_path_buff = web_server_layer.path().join("caddy.json");
        let config_path = config_path_buff.to_str()
            .expect("should provide path to layers directory");

        BuildResultBuilder::new()
            .launch(
                LaunchBuilder::new()
                    .process(
                        ProcessBuilder::new(
                            process_type!("web"), 
                            ["caddy", "run", "--config", config_path]
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
