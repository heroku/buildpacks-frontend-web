use crate::BUILDPACK_NAME;
use commons_ruby::output::build_log::{BuildLog, Logger, StartedLogger};
use commons_ruby::output::fmt;
use commons_ruby::output::fmt::DEBUG_INFO;
use indoc::formatdoc;
use libcnb::TomlFileError;
use std::fmt::Display;
use std::io;
use std::io::stdout;

const SUBMIT_AN_ISSUE: &str = "\
If the issue persists and you think you found a bug in the buildpack then reproduce the issue \
locally with a minimal example and open an issue in the buildpack's GitHub repository with the details.";

#[derive(Debug)]
pub(crate) enum WebsitePublicHTMLBuildpackError {
    Detect(io::Error),
    CannotReadProjectToml(TomlFileError),
    SettingBuildPlanMetadata(toml::ser::Error),
}

pub(crate) fn on_error(error: libcnb::Error<WebsitePublicHTMLBuildpackError>) {
    let logger = BuildLog::new(stdout()).without_buildpack_name();
    match error {
        libcnb::Error::BuildpackError(buildpack_error) => {
            on_buildpack_error(buildpack_error, logger);
        }
        framework_error => on_framework_error(&framework_error, logger),
    }
}

fn on_buildpack_error(error: WebsitePublicHTMLBuildpackError, logger: Box<dyn StartedLogger>) {
    match error {
        WebsitePublicHTMLBuildpackError::Detect(e) => on_detect_error(&e, logger),
        WebsitePublicHTMLBuildpackError::CannotReadProjectToml(e) => on_toml_error(&e, logger),
        WebsitePublicHTMLBuildpackError::SettingBuildPlanMetadata(e) => {
            on_toml_serialization_error(&e, logger);
        }
    }
}

fn on_detect_error(error: &io::Error, logger: Box<dyn StartedLogger>) {
    print_error_details(logger, &error)
        .announce()
        .error(&formatdoc! {"
            Unable to complete buildpack detection.

            An unexpected error occurred while determining if the {buildpack_name} should be \
            run for this application. See the log output above for more information. 
        ", buildpack_name = fmt::value(BUILDPACK_NAME) });
}

fn on_toml_error(error: &TomlFileError, logger: Box<dyn StartedLogger>) {
    print_error_details(logger, &error)
        .announce()
        .error(&formatdoc! {"
            TOML error from {buildpack_name}. 
        ", buildpack_name = fmt::value(BUILDPACK_NAME) });
}

fn on_toml_serialization_error(error: &toml::ser::Error, logger: Box<dyn StartedLogger>) {
    print_error_details(logger, &error)
        .announce()
        .error(&formatdoc! {"
            TOML serialization error from {buildpack_name}. 
        ", buildpack_name = fmt::value(BUILDPACK_NAME) });
}

fn on_framework_error(
    error: &libcnb::Error<WebsitePublicHTMLBuildpackError>,
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
