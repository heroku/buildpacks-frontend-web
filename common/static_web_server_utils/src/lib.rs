use std::path::Path;

use libcnb::{read_toml_file, TomlFileError};
use libherokubuildpack::toml::toml_select_value;

pub fn read_project_config(dir: &Path) -> Result<Option<toml::Value>, TomlFileError> {
    let project_toml_path = dir.join("project.toml");
    let project_toml = if project_toml_path.is_file() {
        read_toml_file::<toml::Value>(project_toml_path)?
    } else {
        toml::Table::new().into()
    };
    let project_config: Option<toml::Value> =
        toml_select_value(vec!["com", "heroku", "static-web-server"], &project_toml).cloned();
    Ok(project_config)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::read_project_config;

    #[test]
    fn reads_project_toml() {
        let project_config = read_project_config(
            PathBuf::from("../../buildpacks/static-web-server/tests/fixtures/custom_doc_root")
                .as_path(),
        )
        .unwrap()
        .expect("TOML value");
        let project_table = project_config.as_table().expect("TOML table");
        assert_eq!(project_table.get("build"), None);
        assert_eq!(
            project_table.get("root"),
            Some(&toml::Value::String(String::from("foobar")))
        );
        assert_eq!(project_table.get("index"), None);
        assert_eq!(project_table.get("headers"), None);
    }

    #[test]
    fn no_project_toml() {
        let project_config = read_project_config(
            PathBuf::from("../../buildpacks/static-web-server/tests/fixtures/no_project_toml")
                .as_path(),
        )
        .unwrap();
        assert_eq!(project_config, None);
    }
}
