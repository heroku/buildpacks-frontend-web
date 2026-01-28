use crate::BUILDPACK_NAME;
use crate::o11y::*;
use bullet_stream::{global::print, Print, style};
use indoc::formatdoc;
use libcnb::TomlFileError;

const DEBUG_INFO: &str = "Debug info";

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
    CannotUnpackCaddyTarball(std::io::Error),
    CannotCreateCaddyInstallationDir(std::io::Error),
    CannotCreateCaddyTarballFile(std::io::Error),
    BuildCommandFailed(std::io::Error),
    CannotCreatWebExecD(std::io::Error),
    CannotInstallEnvAsHtmlData(std::io::Error),
}

pub(crate) struct ErrorMessage {
    message: String,
    error_string: String,
    error_id: String,
}

pub(crate) fn on_error(error: libcnb::Error<StaticWebServerBuildpackError>) {
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

fn buildpack_error_message(error: StaticWebServerBuildpackError) -> ErrorMessage {
    match error {
        StaticWebServerBuildpackError::Download(e) => ErrorMessage {
            message: formatdoc! {"
                Unable to download the static web server for {buildpack_name}. 
            ", buildpack_name = style::value(BUILDPACK_NAME) }, 
            error_string: e.to_string(),
            error_id: "download_error".to_string(),
        },
        StaticWebServerBuildpackError::Json(e) => ErrorMessage {
            message: formatdoc! {"
                JSON error from {buildpack_name}. 
            ", buildpack_name = style::value(BUILDPACK_NAME) }, 
            error_string: e.to_string(),
            error_id: "json_error".to_string(),
        },
        StaticWebServerBuildpackError::CannotUnpackCaddyTarball(e) => ErrorMessage {
            message: formatdoc! {"
                Cannot unpack Caddy tarball for {buildpack_name}
            ", buildpack_name = style::value(BUILDPACK_NAME) }, 
            error_string: e.to_string(),
            error_id: "cannot_unpack_caddy_tarball_error".to_string(),
        },
        StaticWebServerBuildpackError::CannotCreateCaddyInstallationDir(e) =>  ErrorMessage {
            message: formatdoc! {"
                Cannot create Caddy installation directory for {buildpack_name}
            ", buildpack_name = style::value(BUILDPACK_NAME) }, 
            error_string: e.to_string(),
            error_id: "cannot_create_caddy_installation_dir_error".to_string(),
        },
        StaticWebServerBuildpackError::CannotCreateCaddyTarballFile(e) => ErrorMessage {
            message: formatdoc! {"
                Cannot create Caddy tarball file for {buildpack_name}
            ", buildpack_name = style::value(BUILDPACK_NAME) }, 
            error_string: e.to_string(),
            error_id: "cannot_create_caddy_tarball_file_error".to_string(),
        },
        StaticWebServerBuildpackError::BuildCommandFailed(e) => ErrorMessage {
            message: formatdoc! {"
                The custom build command [com.heroku.static-web-server.build] exited with failure. 
            "}, 
            error_string: e.to_string(),
            error_id: "build_command_failed_error".to_string(),
        },
        StaticWebServerBuildpackError::CannotCreatWebExecD(e) => ErrorMessage {
            message: formatdoc! {"
                Cannot create exec.d/web for {buildpack_name}
            ", buildpack_name = style::value(BUILDPACK_NAME) }, 
            error_string: e.to_string(),
            error_id: "cannot_create_exec_d_web_error".to_string(),
        },
        StaticWebServerBuildpackError::CannotInstallEnvAsHtmlData(e) => ErrorMessage {
            message: formatdoc! {"
                Cannot install env-as-html-data (runtime configuration program) for {buildpack_name}
            ", buildpack_name = style::value(BUILDPACK_NAME) }, 
            error_string: e.to_string(),
            error_id: "cannot_install_env_as_html_data_error".to_string(),
        },
        StaticWebServerBuildpackError::CannotParseHerokuWebServerConfiguration(e) => ErrorMessage {
            message: formatdoc! {"
                Cannot parse Heroku web server configuration for {buildpack_name}
            ", buildpack_name = style::value(BUILDPACK_NAME) }, 
            error_string: e.to_string(),
            error_id: "cannot_parse_heroku_web_server_configuration_error".to_string(),
        },
        StaticWebServerBuildpackError::CannotReadProjectToml(e) => ErrorMessage {
            message: formatdoc! {"
                Cannot read project.toml for {buildpack_name}
            ", buildpack_name = style::value(BUILDPACK_NAME) }, 
            error_string: e.to_string(),
            error_id: "cannot_read_project_toml_error".to_string(),
        },
        StaticWebServerBuildpackError::CannotWriteCaddyConfiguration(e) => ErrorMessage {
            message: formatdoc! {"
                Cannot write Caddy configuration for {buildpack_name}
            ", buildpack_name = style::value(BUILDPACK_NAME) }, 
            error_string: e.to_string(),
            error_id: "cannot_write_caddy_configuration_error".to_string(),
        },
    }
}

fn framework_error_message(error: &libcnb::Error<StaticWebServerBuildpackError>) -> ErrorMessage {
    let message = formatdoc! {"
        {buildpack_name} internal error.

        The framework used by this buildpack encountered an unexpected error.
        
        If you can't deploy to Heroku due to this issue, check the official Heroku Status page at \
        status.heroku.com for any ongoing incidents. After all incidents resolve, retry your build.

        {SUBMIT_AN_ISSUE}
    ", buildpack_name = style::value(BUILDPACK_NAME) };

    ErrorMessage {
        message,
        error_string: error.to_string(),
        error_id: "framework_error".to_string(),
    }
}