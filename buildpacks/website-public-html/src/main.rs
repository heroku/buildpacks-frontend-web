mod errors;

use crate::errors::WebsitePublicHTMLBuildpackError;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack};
use libherokubuildpack::log::log_header;

const BUILDPACK_NAME: &str = "Heroku Website (Public HTML) Buildpack";
const PUBLIC_HTML_PATH: &str = "public/index.html";

pub(crate) struct WebsitePublicHTMLBuildpack;

impl Buildpack for WebsitePublicHTMLBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = WebsitePublicHTMLBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let public_html = context
            .app_dir
            .join(PUBLIC_HTML_PATH);
        let public_html_exists = public_html
            .try_exists()
            .map_err(WebsitePublicHTMLBuildpackError::Detect)?;
        println!("Detected path: {} ({})", public_html_exists, public_html.display());

        if public_html_exists {
            DetectResultBuilder::pass().build()
        } else {
            DetectResultBuilder::fail().build()
        }
    }

    fn build(&self, _context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        log_header(BUILDPACK_NAME);
        BuildResultBuilder::new().build()
    }
}

impl From<WebsitePublicHTMLBuildpackError> for libcnb::Error<WebsitePublicHTMLBuildpackError> {
    fn from(value: WebsitePublicHTMLBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

buildpack_main!(WebsitePublicHTMLBuildpack);
