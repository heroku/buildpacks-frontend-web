use crate::BUILDPACK_NAME;
use commons::output::build_log::{BuildLog, Logger, StartedLogger};
use commons::output::fmt;
use commons::output::fmt::DEBUG_INFO;
use indoc::formatdoc;
use libcnb::TomlFileError;
use std::fmt::Display;
use std::io;
use std::io::stdout;

const USE_DEBUG_INFORMATION_AND_RETRY_BUILD: &str = "\
Use the debug information above to troubleshoot and retry your build.";

const SUBMIT_AN_ISSUE: &str = "\
If the issue persists and you think you found a bug in the buildpack then reproduce the issue \
locally with a minimal example and open an issue in the buildpack's GitHub repository with the details.";

#[derive(Debug)]
pub(crate) enum StaticWebServerBuildpackError {
    Download(libherokubuildpack::download::DownloadError),
    JSON(serde_json::Error),
    Message(String),
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
        StaticWebServerBuildpackError::JSON(e) => on_json_error(&e, logger),
        StaticWebServerBuildpackError::Message(m) => on_message_error(&m, logger),
    }
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

fn on_message_error(message: &String, logger: Box<dyn StartedLogger>) {
    print_error_details(logger, &message)
        .announce()
        .error(&formatdoc! {"
            Error during build of {buildpack_name}.
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
