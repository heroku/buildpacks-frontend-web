#![allow(unused_crate_dependencies)]
use std::{env, path::Path};

use env_as_html_data::{env_as_html_data, HtmlRewritten};

pub const ALLOWED_FILESYSTEM_ROOT: &str = "/workspace";

fn main() {
    let command_env: std::collections::HashMap<String, String> = env::vars().collect();

    let file_paths: Vec<&str> = command_env.get("ENV_AS_HTML_DATA_TARGET_FILES").or_else(|| {
        eprintln!("Runtime configuration failed: env-as-html-data requires comma-delimited list of target files, the paths of the HTML documents to process. Set with environment variable: ENV_AS_HTML_DATA_TARGET_FILES. (This should be automatically set during CNB build.)");
        std::process::exit(1);
    }).map(|v| v.split(',').collect()).expect("should exit failure when none");

    for file_path in file_paths {
        let trimmed_file_path = file_path.trim();
        let canonical_path = Path::new(trimmed_file_path).canonicalize();
        let fp = match canonical_path {
            Err(e) => {
                eprintln!(
                    "Runtime configuration skipping '{trimmed_file_path}' because the path is invalid: {e}"
                );
                continue;
            }
            Ok(path_buf) => path_buf.clone(),
        };
        if !fp.starts_with(ALLOWED_FILESYSTEM_ROOT) {
            eprintln!(
                "Runtime configuration skipping '{}' because it is not in '{}'.",
                fp.display(),
                ALLOWED_FILESYSTEM_ROOT
            );
            continue;
        }
        match env_as_html_data(&command_env, &fp.clone()) {
            Err(e) => {
                eprintln!("Runtime configuration failed: {e:?}");
                std::process::exit(1);
            }
            Ok(HtmlRewritten::Yes) => {
                eprintln!("Runtime configuration written into {file_path}");
            }
            Ok(HtmlRewritten::No) => {
                eprintln!("No runtime configuration is set, '{file_path}' unchanged");
            }
        }
    }
}
