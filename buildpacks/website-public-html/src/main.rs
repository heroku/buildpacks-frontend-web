mod errors;
mod o11y;

use crate::errors::{on_error, WebsitePublicHTMLBuildpackError};
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::{BuildPlanBuilder, Require};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack, Error};
use libherokubuildpack::log::log_header;
use static_web_server_utils::read_project_config;

// Silence unused dependency warning for
// dependencies only used in tests
#[cfg(test)]
use libcnb_test as _;
#[cfg(test)]
use test_support as _;

const BUILDPACK_NAME: &str = "Heroku Website (Public HTML) Buildpack";
const DEFAULT_ROOT: &str = "public";
const DEFAULT_INDEX: &str = "index.html";

pub(crate) struct WebsitePublicHTMLBuildpack;

impl Buildpack for WebsitePublicHTMLBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = WebsitePublicHTMLBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let project_config = read_project_config(context.app_dir.as_ref())
            .map_err(WebsitePublicHTMLBuildpackError::CannotReadProjectToml)?;
        let (root, index) = match project_config {
            Some(v) => {
                v.as_table()
                    .map_or((DEFAULT_ROOT.to_string(), DEFAULT_INDEX.to_string()), |t| {
                        let r = t
                            .get("root")
                            .map(|a| a.as_str().unwrap_or_default())
                            .or(Some(DEFAULT_ROOT))
                            .unwrap_or_default()
                            .to_string();
                        let i = t
                            .get("index")
                            .map(|a| a.as_str().unwrap_or_default())
                            .or(Some(DEFAULT_INDEX))
                            .unwrap_or_default()
                            .to_string();
                        (r, i)
                    })
            }
            None => (DEFAULT_ROOT.to_string(), DEFAULT_INDEX.to_string()),
        };

        // Check that the root + index exists in the workspace.
        let index_page = context.app_dir.join(&root).join(&index);
        let index_page_exists = index_page
            .try_exists()
            .map_err(WebsitePublicHTMLBuildpackError::Detect)?;

        // Set Build Plan metadata for Static Web Server.
        let mut static_web_server_req = Require::new("static-web-server");
        let mut metadata_table = toml::Table::new();
        metadata_table.insert("root".to_string(), toml::Value::String(root));
        metadata_table.insert("index".to_string(), toml::Value::String(index));
        static_web_server_req
            .metadata(metadata_table)
            .map_err(WebsitePublicHTMLBuildpackError::SettingBuildPlanMetadata)?;
        let plan_builder = BuildPlanBuilder::new().requires(static_web_server_req);

        if index_page_exists {
            DetectResultBuilder::pass()
                .build_plan(plan_builder.build())
                .build()
        } else {
            DetectResultBuilder::fail().build()
        }
    }

    fn build(&self, _context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        log_header(BUILDPACK_NAME);
        BuildResultBuilder::new().build()
    }

    fn on_error(&self, error: Error<Self::Error>) {
        on_error(error);
    }
}

impl From<WebsitePublicHTMLBuildpackError> for libcnb::Error<WebsitePublicHTMLBuildpackError> {
    fn from(value: WebsitePublicHTMLBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

buildpack_main!(WebsitePublicHTMLBuildpack);
