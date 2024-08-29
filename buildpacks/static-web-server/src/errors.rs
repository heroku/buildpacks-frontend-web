use crate::BUILDPACK_NAME;
use commons::output::build_log::{BuildLog, Logger, StartedLogger};
use commons::output::fmt;
use commons::output::fmt::DEBUG_INFO;
use indoc::formatdoc;
use libcnb::TomlFileError;
use std::fmt::Display;
use std::io::stdout;

const SUBMIT_AN_ISSUE: &str = "\
If the issue persists and you think you found a bug in the buildpack then reproduce the issue \
locally with a minimal example and open an issue in the buildpack's GitHub repository with the details.";

#[derive(Debug)]
pub(crate) enum StaticWebServerBuildpackError {
    Download(libherokubuildpack::download::DownloadError),
    Json(serde_json::Error),
    CannotParseHerokuWebServerConfiguration(toml::de::Error),
    CannotReadProjectToml(TomlFileError),
    CannotWriteCaddyConfiguration(std::io::Error),
    CannotReadCustom404File(std::io::Error),
    CannotUnpackCaddyTarball(std::io::Error),
    CannotCreateCaddyInstallationDir(std::io::Error),
    CannotCreateCaddyTarballFile(std::io::Error),
}

pub(crate) fn on_error(error: libcnb::Error<StaticWebServerBuildpackError>) {
    let logger = BuildLog::new(stdout()).without_buildpack_name();
    match error {
        libcnb::Error::BuildpackError(buildpack_error) => {
            on_buildpack_error(buildpack_error, logger);
        }
        framework_error => on_framework_error(&framework_error, logger),
    }
}

fn on_buildpack_error(error: StaticWebServerBuildpackError, logger: Box<dyn StartedLogger>) {
    match error {
        StaticWebServerBuildpackError::Download(e) => on_download_error(&e, logger),
        StaticWebServerBuildpackError::Json(e) => on_json_error(&e, logger),
        StaticWebServerBuildpackError::CannotReadProjectToml(e) => on_toml_error(&e, logger),
        StaticWebServerBuildpackError::CannotParseHerokuWebServerConfiguration(e) => on_config_error(&e, logger),
        StaticWebServerBuildpackError::CannotWriteCaddyConfiguration(error)
        | StaticWebServerBuildpackError::CannotReadCustom404File(error)
        | StaticWebServerBuildpackError::CannotUnpackCaddyTarball(error)
        | StaticWebServerBuildpackError::CannotCreateCaddyInstallationDir(error)
        | StaticWebServerBuildpackError::CannotCreateCaddyTarballFile(error) => {
            on_unexpected_io_error(&error, logger);
        }
    }
}

fn on_unexpected_io_error(error: &std::io::Error, logger: Box<dyn StartedLogger>) {
    print_error_details(logger, &error)
        .announce()
        .error(&formatdoc! {"
        Unexpected IO Error

        An unexpected IO error occurred. Please try again.

        {SUBMIT_AN_ISSUE}
    "});
}

fn on_download_error(
    error: &libherokubuildpack::download::DownloadError,
    logger: Box<dyn StartedLogger>,
) {
    print_error_details(logger, &error)
        .announce()
        .error(&formatdoc! {"
            Unable to download the static web server for {buildpack_name}. 
        ", buildpack_name = fmt::value(BUILDPACK_NAME) });
}

fn on_json_error(error: &serde_json::Error, logger: Box<dyn StartedLogger>) {
    print_error_details(logger, &error)
        .announce()
        .error(&formatdoc! {"
            JSON error from {buildpack_name}. 
        ", buildpack_name = fmt::value(BUILDPACK_NAME) });
}

fn on_toml_error(error: &TomlFileError, logger: Box<dyn StartedLogger>) {
    print_error_details(logger, &error)
        .announce()
        .error(&formatdoc! {"
            TOML error from {buildpack_name}. 
        ", buildpack_name = fmt::value(BUILDPACK_NAME) });
}

fn on_config_error(error: &toml::de::Error, logger: Box<dyn StartedLogger>) {
    print_error_details(logger, &error)
        .announce()
        .error(&formatdoc! {"
            Configuration error from {buildpack_name}. 
        ", buildpack_name = fmt::value(BUILDPACK_NAME) });
}

fn on_framework_error(
    error: &libcnb::Error<StaticWebServerBuildpackError>,
    logger: Box<dyn StartedLogger>,
) {
    print_error_details(logger, &error)
        .announce()
        .error(&formatdoc! {"
            {buildpack_name} internal error.

            The framework used by this buildpack encountered an unexpected error.
            
            If you can't deploy to Heroku due to this issue, check the official Heroku Status page at \
            status.heroku.com for any ongoing incidents. After all incidents resolve, retry your build.

            {SUBMIT_AN_ISSUE}
        ", buildpack_name = fmt::value(BUILDPACK_NAME) });
}

fn print_error_details(
    logger: Box<dyn StartedLogger>,
    error: &impl Display,
) -> Box<dyn StartedLogger> {
    logger
        .section(DEBUG_INFO)
        .step(&error.to_string())
        .end_section()
}
