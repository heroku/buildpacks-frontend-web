use crate::BUILDPACK_NAME;
use bullet_stream::{global::print, style};
use indoc::formatdoc;
use libcnb::TomlFileError;
use std::fmt::Display;

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

pub(crate) fn on_error(error: libcnb::Error<StaticWebServerBuildpackError>) {
    match error {
        libcnb::Error::BuildpackError(buildpack_error) => {
            on_buildpack_error(buildpack_error);
        }
        framework_error => on_framework_error(&framework_error),
    }
}

fn on_buildpack_error(error: StaticWebServerBuildpackError) {
    match error {
        StaticWebServerBuildpackError::Download(e) => on_download_error(&e),
        StaticWebServerBuildpackError::Json(e) => on_json_error(&e),
        StaticWebServerBuildpackError::CannotReadProjectToml(e) => on_toml_error(&e),
        StaticWebServerBuildpackError::CannotParseHerokuWebServerConfiguration(e) => {
            on_config_error(&e);
        }
        StaticWebServerBuildpackError::CannotWriteCaddyConfiguration(error) => {
            print_error_details(&error);
            print::error(formatdoc! {"
                Cannot write Caddy configuration for {buildpack_name}
            ", buildpack_name = style::value(BUILDPACK_NAME) });
        }
        StaticWebServerBuildpackError::CannotUnpackCaddyTarball(error) => {
            print_error_details(&error);
            print::error(formatdoc! {"
                Cannot unpack Caddy tarball for {buildpack_name}
            ", buildpack_name = style::value(BUILDPACK_NAME) });
        }
        StaticWebServerBuildpackError::CannotCreateCaddyInstallationDir(error) => {
            print_error_details(&error);
            print::error(formatdoc! {"
                Cannot create Caddy installation directory for {buildpack_name}
            ", buildpack_name = style::value(BUILDPACK_NAME) });
        }
        StaticWebServerBuildpackError::CannotCreateCaddyTarballFile(error) => {
            print_error_details(&error);
            print::error(formatdoc! {"
                Cannot create Caddy tarball file for {buildpack_name}
            ", buildpack_name = style::value(BUILDPACK_NAME) });
        }
        StaticWebServerBuildpackError::BuildCommandFailed(e) => on_build_command_error(&e),
        StaticWebServerBuildpackError::CannotCreatWebExecD(error) => {
            print_error_details(&error);
            print::error(formatdoc! {"
                Cannot create exec.d/web for {buildpack_name}
            ", buildpack_name = style::value(BUILDPACK_NAME) });
        }
        StaticWebServerBuildpackError::CannotInstallEnvAsHtmlData(error) => {
            print_error_details(&error);
            print::error(formatdoc! {"
                Cannot install env-as-html-data (runtime configuration program) for {buildpack_name}
            ", buildpack_name = style::value(BUILDPACK_NAME) });
        }
    }
}

fn on_download_error(error: &libherokubuildpack::download::DownloadError) {
    print_error_details(&error);
    print::error(formatdoc! {"
        Unable to download the static web server for {buildpack_name}. 
    ", buildpack_name = style::value(BUILDPACK_NAME) });
}

fn on_json_error(error: &serde_json::Error) {
    print_error_details(&error);
    print::error(formatdoc! {"
        JSON error from {buildpack_name}. 
    ", buildpack_name = style::value(BUILDPACK_NAME) });
}

fn on_toml_error(error: &TomlFileError) {
    print_error_details(&error);
    print::error(formatdoc! {"
        TOML error from {buildpack_name}. 
    ", buildpack_name = style::value(BUILDPACK_NAME) });
}

fn on_config_error(error: &toml::de::Error) {
    print_error_details(&error);
    print::error(formatdoc! {"
        Configuration error from {buildpack_name}. 
    ", buildpack_name = style::value(BUILDPACK_NAME) });
}

fn on_build_command_error(error: &std::io::Error) {
    print_error_details(&error);
    print::error(formatdoc! {"
        The custom build command [com.heroku.static-web-server.build] exited with failure. 
    "});
}

fn on_framework_error(error: &libcnb::Error<StaticWebServerBuildpackError>) {
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
