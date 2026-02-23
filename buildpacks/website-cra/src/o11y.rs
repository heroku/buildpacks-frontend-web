use const_format::formatcp;

const NAMESPACE: &str = "cnb.website-cra";

const ERROR: &str = formatcp!("{NAMESPACE}.error");
pub(crate) const ERROR_ID: &str = formatcp!("{ERROR}.id");
pub(crate) const ERROR_MESSAGE: &str = formatcp!("{ERROR}.message");
