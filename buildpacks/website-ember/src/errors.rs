use crate::BUILDPACK_NAME;
use bullet_stream::{global::print, style};
use indoc::formatdoc;
use std::fmt::Display;
use std::io;

const DEBUG_INFO: &str = "Debug info";

const SUBMIT_AN_ISSUE: &str = "\
If the issue persists and you think you found a bug in the buildpack then reproduce the issue \
locally with a minimal example and open an issue in the buildpack's GitHub repository with the details.";

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum WebsiteEmberBuildpackError {
    Detect(io::Error),
    ReadPackageJson(io::Error),
    ParsePackageJson(serde_json::Error),
    SettingBuildPlanMetadata(toml::ser::Error),
}

pub(crate) fn on_error(error: libcnb::Error<WebsiteEmberBuildpackError>) {
    match error {
        libcnb::Error::BuildpackError(buildpack_error) => on_buildpack_error(buildpack_error),
        framework_error => on_framework_error(&framework_error),
    }
}

fn on_buildpack_error(error: WebsiteEmberBuildpackError) {
    match error {
        WebsiteEmberBuildpackError::Detect(e) => on_detect_error(&e),
        WebsiteEmberBuildpackError::ReadPackageJson(e) => on_read_package_json_error(&e),
        WebsiteEmberBuildpackError::ParsePackageJson(e) => on_parse_package_json_error(&e),
        WebsiteEmberBuildpackError::SettingBuildPlanMetadata(e) => on_toml_serialization_error(&e),
    }
}

fn on_detect_error(error: &io::Error) {
    print_error_details(&error);
    print::error(formatdoc! {"
        Unable to complete buildpack detection.

        An unexpected error occurred while determining if the {buildpack_name} should be \
        run for this application. See the log output above for more information. 
    ", buildpack_name = style::value(BUILDPACK_NAME) });
}

fn on_read_package_json_error(error: &io::Error) {
    print_error_details(&error);
    print::error(formatdoc! {"
        Error reading package.json from {buildpack_name}.
    ", buildpack_name = style::value(BUILDPACK_NAME) });
}

fn on_parse_package_json_error(error: &serde_json::Error) {
    print_error_details(&error);
    print::error(formatdoc! {"
        Error parsing package.json from {buildpack_name}.
    ", buildpack_name = style::value(BUILDPACK_NAME) });
}

fn on_toml_serialization_error(error: &toml::ser::Error) {
    print_error_details(&error);
    print::error(formatdoc! {"
        TOML serialization error from {buildpack_name}. 
    ", buildpack_name = style::value(BUILDPACK_NAME) });
}

fn on_framework_error(error: &libcnb::Error<WebsiteEmberBuildpackError>) {
    print_error_details(&error);
    print::error(formatdoc! {"
        {buildpack_name} internal error.

        The framework used by this buildpack encountered an unexpected error.
        
        If you can't deploy to Heroku due to this issue, check the official Heroku Status page at \
        status.heroku.com for any ongoing incidents. After all incidents resolve, retry your build.

        {SUBMIT_AN_ISSUE}
    ", buildpack_name = style::value(BUILDPACK_NAME) });
}

fn print_error_details(error: &impl Display) {
    print::bullet(style::important(DEBUG_INFO));
    print::bullet(error.to_string());
}
