use std::{env, path::PathBuf};

use env_as_html_data::env_as_html_data;

fn main() {
    let command_env: std::collections::HashMap<String, String> = env::vars().collect();

    let file_paths: Vec<&str> = command_env.get("ENV_AS_HTML_DATA_TARGET_FILES").or_else(|| {
        eprintln!("Runtime configuration failed: env-as-html-data requires comma-delimited list of target files, the paths of the HTML documents to process. Set with environment variable: ENV_AS_HTML_DATA_TARGET_FILES. (This should be automatically set during CNB build.)");
        std::process::exit(1);
    }).map(|v| v.split(',').collect()).expect("should exit failure when none");

    for file_path in file_paths {
        match env_as_html_data(&command_env, &PathBuf::from(file_path.trim())) {
            Err(e) => {
                eprintln!("Runtime configuration failed: {e:?}");
                std::process::exit(1);
            }
            Ok(true) => {
                eprintln!("Runtime configuration written into {file_path}");
            }
            _ => (),
        }
    }
}
