use const_format::formatcp;

const NAMESPACE: &str = "cnb.website-public-html";
const DETECT: &str = formatcp!("{NAMESPACE}.detect");
pub(crate) const DETECT_PROVIDES_WEBSITE_PUBLIC_HTML: &str =
    formatcp!("{DETECT}.provides_website_public_html");
pub(crate) const DETECT_REQUIRES_WEBSITE_PUBLIC_HTML: &str =
    formatcp!("{DETECT}.requires_website_public_html");

const ERROR: &str = formatcp!("{NAMESPACE}.error");
pub(crate) const ERROR_ID: &str = formatcp!("{ERROR}.id");
pub(crate) const ERROR_MESSAGE: &str = formatcp!("{ERROR}.message");
