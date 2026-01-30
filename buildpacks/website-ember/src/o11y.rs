use const_format::formatcp;

const NAMESPACE: &str = "cnb.website-ember";
const DETECT: &str = formatcp!("{NAMESPACE}.detect");
pub(crate) const DETECT_PROVIDES_WEBSITE_EMBER: &str = formatcp!("{DETECT}.provides_website_ember");
pub(crate) const DETECT_REQUIRES_WEBSITE_EMBER: &str = formatcp!("{DETECT}.requires_website_ember");

const ERROR: &str = formatcp!("{NAMESPACE}.error");
pub(crate) const ERROR_ID: &str = formatcp!("{ERROR}.id");
pub(crate) const ERROR_MESSAGE: &str = formatcp!("{ERROR}.message");
