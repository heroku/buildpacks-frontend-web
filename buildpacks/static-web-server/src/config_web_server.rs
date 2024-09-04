use crate::caddy_config::CaddyConfig;
use crate::heroku_web_server_config::HerokuWebServerConfig;
use crate::{StaticWebServerBuildpack, StaticWebServerBuildpackError};
use libcnb::data::layer_name;
use libcnb::layer::LayerRef;
use libcnb::read_toml_file;
use libcnb::{build::BuildContext, layer::UncachedLayerDefinition};
use libherokubuildpack::log::log_info;
use libherokubuildpack::toml::toml_select_value;
use toml::Table;
use std::fs;
use std::process::{Command, Output};

pub(crate) fn config_web_server(
    context: &BuildContext<StaticWebServerBuildpack>,
) -> Result<LayerRef<StaticWebServerBuildpack, (), ()>, libcnb::Error<StaticWebServerBuildpackError>>
{
    let configuration_layer = context.uncached_layer(
        layer_name!("configuration"),
        UncachedLayerDefinition {
            build: false,
            launch: true,
        },
    )?;

    // Load a table of Build Plan [requires.metadata] from context.
    // When a key is defined multiple times, the last one wins.
    let mut build_plan_config = Table::new();
    context.buildpack_plan.entries.iter().for_each(|e| {
        e.metadata.iter().for_each(|(k, v)| {
            build_plan_config.insert(k.to_owned(), v.to_owned());
        });
    });

    // Load the table of [com.heroku.static-web-server] from project.toml
    let project_toml_path = context.app_dir.join("project.toml");
    let project_toml = if project_toml_path.is_file() {
        read_toml_file::<toml::Value>(project_toml_path)
            .map_err(StaticWebServerBuildpackError::CannotReadProjectToml)?
    } else {
        toml::Table::new().into()
    };
    let project_config: Option<&toml::Value> = 
        toml_select_value(vec!["com", "heroku", "static-web-server"], &project_toml);

    let heroku_config = generate_config_with_inheritance(project_config, build_plan_config)?;

    let build_command_opt = heroku_config.build.clone();

    // Transform web server config to Caddy native JSON config
    let caddy_config = CaddyConfig::try_from(heroku_config)?;
    let caddy_config_json =
        serde_json::to_string(&caddy_config).map_err(StaticWebServerBuildpackError::Json)?;
    let config_path = configuration_layer.path().join("caddy.json");
    fs::write(config_path, caddy_config_json)
        .map_err(StaticWebServerBuildpackError::CannotWriteCaddyConfiguration)?;

    // Execute the optional build command
    build_command_opt.map(|e| -> Result<Output, StaticWebServerBuildpackError> {
        log_info(format!("Executing build command: {e:#?}"));
        let mut cmd = Command::new(e.command.clone());
        e.args.clone().map(|v| cmd.args(v) );
        let output = cmd
            .output()
            .map_err(StaticWebServerBuildpackError::BuildCommandFailed)?;

        log_info(format!("status: {}", output.status));
        log_info(format!("stdout: {}", String::from_utf8_lossy(&output.stdout)));
        log_info(format!("stderr: {}", String::from_utf8_lossy(&output.stderr)));

        Ok(output)
    });

    Ok(configuration_layer)
}

fn generate_config_with_inheritance(
    project_config: Option<&toml::Value>,
    config_to_inherit: toml::map::Map<String, toml::Value>
) -> Result<HerokuWebServerConfig, libcnb::Error<StaticWebServerBuildpackError>> {
    // Default config is from the Build Plan metadata or empty.
    let default_config: HerokuWebServerConfig = config_to_inherit
        .clone()
        .try_into()
        .map_err(StaticWebServerBuildpackError::CannotParseHerokuWebServerConfiguration)?;

    let heroku_config: HerokuWebServerConfig = project_config.map_or(
        Ok(default_config),
        |table| {
            let mut config_from_project: toml::Table = table
                .clone()
                .try_into()
                .unwrap();

            config_to_inherit.iter().for_each(|(bpk, bpv)| {
                if !config_from_project.contains_key(bpk) {
                    config_from_project.insert(bpk.to_owned(), bpv.to_owned());
                }
            });
            config_from_project.try_into()
                .map_err(StaticWebServerBuildpackError::CannotParseHerokuWebServerConfiguration)
        },
    )?;
        
    Ok(heroku_config)
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use toml::toml;

    use crate::config_web_server::generate_config_with_inheritance;

    #[test]
    fn generate_config_default() {
        let inherit_config = toml::Table::new();

        let parsed_config = generate_config_with_inheritance(None, inherit_config)
            .unwrap();
        assert_eq!(parsed_config.build, None);
        assert_eq!(parsed_config.root, None);
        assert_eq!(parsed_config.index, None);
        assert_eq!(parsed_config.headers, None);
    }

    #[test]
    fn generate_config_with_project_toml() {
        let project_config: toml::Value = toml! {
            root = "files/web"
        }.into();
        let inherit_config = toml::Table::new();

        let parsed_config = generate_config_with_inheritance(Some(&project_config), inherit_config)
            .unwrap();
        assert_eq!(parsed_config.build, None);
        assert_eq!(parsed_config.root, Some(PathBuf::from("files/web")));
        assert_eq!(parsed_config.index, None);
        assert_eq!(parsed_config.headers, None);
    }

    #[test]
    fn generate_config_with_build_plan() {
        let mut inherit_config = toml::Table::new();
        inherit_config.insert("root".to_string(), "www".to_string().into());

        let parsed_config = generate_config_with_inheritance(None, inherit_config)
            .unwrap();
        assert_eq!(parsed_config.build, None);
        assert_eq!(parsed_config.root, Some(PathBuf::from("www")));
        assert_eq!(parsed_config.index, None);
        assert_eq!(parsed_config.headers, None);
    }

    #[test]
    fn generate_config_with_project_precedence() {
        let project_config: toml::Value = toml! {
            root = "value/with/precedence"
        }.into();
        let mut inherit_config = toml::Table::new();
        inherit_config.insert("root".to_string(), "value/should/be/overriden".to_string().into());
        inherit_config.insert("index".to_string(), "main.html".to_string().into());

        let parsed_config = generate_config_with_inheritance(Some(&project_config), inherit_config)
            .unwrap();
        assert_eq!(parsed_config.build, None);
        assert_eq!(parsed_config.root, Some(PathBuf::from("value/with/precedence")));
        assert_eq!(parsed_config.index, Some(String::from("main.html")));
        assert_eq!(parsed_config.headers, None);
    }
}
