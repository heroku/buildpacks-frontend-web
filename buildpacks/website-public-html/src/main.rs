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
    // The CNB platform this buildpack targets, usually `GenericPlatform`. See the CNB spec for
    // more information about platforms:
    // https://github.com/buildpacks/spec/blob/main/buildpack.md
    type Platform = GenericPlatform;

    // The type for the metadata of the buildpack itself. This is the data found in the
    // `[metadata]` section of your buildpack's `buildpack.toml`. The framework will automatically
    // try to parse it into the specified type. This example buildpack uses GenericMetadata which
    // provides low-level access to the TOML table.
    type Metadata = GenericMetadata;

    // The error type for this buildpack. Buildpack authors usually implement an enum with
    // specific errors that can happen during buildpack execution. This error type should
    // only contain error specific to this buildpack, such as `CouldNotExecuteMaven` or
    // `InvalidGemfileLock`. This example buildpack uses `GenericError` which means this buildpack
    // does not specify any errors.
    //
    // Common errors that happen during buildpack execution such as I/O errors while
    // writing CNB TOML files are handled by libcnb.rs itself.
    type Error = WebsitePublicHTMLBuildpackError;

    // This method will be called when the CNB lifecycle executes the detect phase (`bin/detect`).
    // Use the `DetectContext` to access CNB data such as the operating system this buildpack is currently
    // executed on, the app directory and similar things. When using libcnb.rs, you never have
    // to read environment variables or read/write files to disk to interact with the CNB lifecycle.
    //
    // One example of this is the return type of this method. `DetectResult` encapsulates the
    // required exit code as well as the data written to the build plan. libcnb.rs will,
    // according to the returned value, handle both writing the build plan and exiting with
    // the correct status code for you.
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

    // Similar to detect, this method will be called when the CNB lifecycle executes the
    // build phase (`bin/build`).
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

// Implements the main function and wires up the framework for the given buildpack.
buildpack_main!(WebsitePublicHTMLBuildpack);
