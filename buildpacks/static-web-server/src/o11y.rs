use const_format::formatcp;

const NAMESPACE: &str = "cnb.static-web-server";
const DETECT: &str = formatcp!("{NAMESPACE}.detect");
pub(crate) const DETECT_PROVIDES_STATIC_WEB_SERVER: &str = formatcp!("{DETECT}.provides_static_web_server");
pub(crate) const DETECT_REQUIRES_STATIC_WEB_SERVER: &str = formatcp!("{DETECT}.requires_static_web_server");

const ERROR: &str = formatcp!("{NAMESPACE}.error");
pub(crate) const ERROR_ID: &str = formatcp!("{ERROR}.id");
pub(crate) const ERROR_MESSAGE: &str = formatcp!("{ERROR}.message");