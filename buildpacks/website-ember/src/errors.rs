use crate::BUILDPACK_NAME;
use crate::o11y::*;
use bullet_stream::{global::print, Print, style};
use indoc::formatdoc;
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

pub(crate) struct ErrorMessage {
    message: String,
    error_string: String,
    error_id: String,
}

pub(crate) fn on_error(error: libcnb::Error<WebsiteEmberBuildpackError>) {
    let error_message = match error {
        libcnb::Error::BuildpackError(buildpack_error) => buildpack_error_message(buildpack_error),
        framework_error => framework_error_message(&framework_error),
    };

    let output = Print::new(vec![]).without_header()
        .bullet(style::important(DEBUG_INFO))
        .sub_bullet(error_message.error_string)
        .done()
        .error(error_message.message);

    let output_string = String::from_utf8_lossy(&output).to_string();

    tracing::error!(
        { ERROR_ID } = error_message.error_id,
        { ERROR_MESSAGE } = output_string,
        "error"
    );
    print::plain(output_string);
    eprintln!();
}

fn buildpack_error_message(error: WebsiteEmberBuildpackError) -> ErrorMessage {
    match error {
        WebsiteEmberBuildpackError::Detect(e) => ErrorMessage {
            message: formatdoc! {"
                Unable to complete buildpack detection.
            "}, 
            error_string: e.to_string(),
            error_id: "detect_error".to_string(),
        },
        WebsiteEmberBuildpackError::ReadPackageJson(e) => ErrorMessage {
            message: formatdoc! {"
                Error reading package.json from {buildpack_name}.
            ", buildpack_name = style::value(BUILDPACK_NAME) }, 
            error_string: e.to_string(),
            error_id: "read_package_json_error".to_string(),
        },
        WebsiteEmberBuildpackError::ParsePackageJson(e) => ErrorMessage {
            message: formatdoc! {"
                Error parsing package.json from {buildpack_name}.
            ", buildpack_name = style::value(BUILDPACK_NAME) }, 
            error_string: e.to_string(),
            error_id: "parse_package_json_error".to_string(),
        },
        WebsiteEmberBuildpackError::SettingBuildPlanMetadata(e) => ErrorMessage {
            message: formatdoc! {"
                Error setting build plan metadata from {buildpack_name}.
            ", buildpack_name = style::value(BUILDPACK_NAME) }, 
            error_string: e.to_string(),
            error_id: "setting_build_plan_metadata_error".to_string(),
        },
    }
}

fn framework_error_message(error: &libcnb::Error<WebsiteEmberBuildpackError>) -> ErrorMessage {
    ErrorMessage {
        message: formatdoc! {"
            {buildpack_name} internal error.
        ", buildpack_name = style::value(BUILDPACK_NAME) }, 
        error_string: error.to_string(),
        error_id: "website_ember_buildpack_error".to_string(),
    }
}
