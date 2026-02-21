use const_format::formatcp;

const NAMESPACE: &str = "cnb.website-cra";
const DETECT: &str = formatcp!("{NAMESPACE}.detect");
pub(crate) const DETECT_PROVIDES_WEBSITE_CRA: &str = formatcp!("{DETECT}.provides_website_cra");
pub(crate) const DETECT_REQUIRES_WEBSITE_CRA: &str = formatcp!("{DETECT}.requires_website_cra");

const ERROR: &str = formatcp!("{NAMESPACE}.error");
pub(crate) const ERROR_ID: &str = formatcp!("{ERROR}.id");
pub(crate) const ERROR_MESSAGE: &str = formatcp!("{ERROR}.message");
