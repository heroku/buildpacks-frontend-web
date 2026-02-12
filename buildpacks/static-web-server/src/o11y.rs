use const_format::formatcp;

const NAMESPACE: &str = "cnb.static-web-server";
const DETECT: &str = formatcp!("{NAMESPACE}.detect");
pub(crate) const DETECT_PROVIDES_STATIC_WEB_SERVER: &str =
    formatcp!("{DETECT}.provides_static_web_server");
pub(crate) const DETECT_REQUIRES_STATIC_WEB_SERVER: &str =
    formatcp!("{DETECT}.requires_static_web_server");

const INSTALLATION: &str = formatcp!("{NAMESPACE}.installation");
pub(crate) const INSTALLATION_WEB_SERVER_NAME: &str = formatcp!("{INSTALLATION}.web_server_name");
pub(crate) const INSTALLATION_WEB_SERVER_VERSION: &str =
    formatcp!("{INSTALLATION}.web_server_version");

const CONFIG: &str = formatcp!("{NAMESPACE}.config");
pub(crate) const CONFIG_CADDY_SERVER_OPTS_BASIC_AUTH: &str =
    formatcp!("{CONFIG}.caddy_server_opts_basic_auth");
pub(crate) const CONFIG_CADDY_SERVER_OPTS_CLEAN_URLS: &str =
    formatcp!("{CONFIG}.caddy_server_opts_clean_urls");
pub(crate) const CONFIG_CADDY_SERVER_OPTS_TEMPLATES: &str =
    formatcp!("{CONFIG}.caddy_server_opts_templates");
pub(crate) const CONFIG_CADDY_SERVER_OPTS_ACCESS_LOGS: &str =
    formatcp!("{CONFIG}.caddy_server_opts_access_logs");
pub(crate) const CONFIG_DOC_ROOT_PATH: &str = formatcp!("{CONFIG}.doc_root_path");
pub(crate) const CONFIG_DOC_INDEX: &str = formatcp!("{CONFIG}.doc_index");
pub(crate) const CONFIG_BUILD_COMMAND: &str = formatcp!("{CONFIG}.build_command");
pub(crate) const CONFIG_RUNTIME_CONFIG_ENABLED: &str = formatcp!("{CONFIG}.runtime_config_enabled");
pub(crate) const CONFIG_ERROR_404_FILE_PATH: &str = formatcp!("{CONFIG}.error_404_file_path");
pub(crate) const CONFIG_ERROR_404_STATUS_CODE: &str = formatcp!("{CONFIG}.error_404_status_code");
pub(crate) const CONFIG_RESPONSE_HEADERS_ENABLED: &str =
    formatcp!("{CONFIG}.response_headers_enabled");

const ERROR: &str = formatcp!("{NAMESPACE}.error");
pub(crate) const ERROR_ID: &str = formatcp!("{ERROR}.id");
pub(crate) const ERROR_MESSAGE: &str = formatcp!("{ERROR}.message");
pub(crate) const ERROR_WEB_SERVER_NAME: &str = formatcp!("{ERROR}.web_server_name");
pub(crate) const ERROR_WEB_SERVER_VERSION: &str = formatcp!("{ERROR}.web_server_version");
