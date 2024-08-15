use libcnb::{build::BuildContext, layer::UncachedLayerDefinition};
use libcnb::data::layer_name;
use libherokubuildpack::log::log_info;

use crate::{WebsitePublicHTMLBuildpack, WebsitePublicHTMLBuildpackError, DOC_ROOT};

pub(crate) fn import_doc_root(
    context: &BuildContext<WebsitePublicHTMLBuildpack>,
) -> Result<(), libcnb::Error<WebsitePublicHTMLBuildpackError>> {
    
    let layer_ref = context.uncached_layer(
        layer_name!("doc_root"),
        UncachedLayerDefinition {
            build: true,
            launch: true,
        },
    )?;

    let public_html_source = &context
        .app_dir
        .join(DOC_ROOT);
    let doc_root_parent = layer_ref.path();
    log_info(format!("Importing doc root: {} to {}", public_html_source.display(), doc_root_parent.display()));

    let copy_options = fs_extra::dir::CopyOptions::new();
    fs_extra::dir::copy(public_html_source, doc_root_parent, &copy_options)
        .map_err(WebsitePublicHTMLBuildpackError::DocRoot)?;

    Ok(())
}

