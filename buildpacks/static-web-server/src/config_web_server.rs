use crate::caddy_config::CaddyConfig;
use crate::heroku_web_server_config::HerokuWebServerConfig;
use crate::{StaticWebServerBuildpack, StaticWebServerBuildpackError};
use libcnb::data::layer_name;
use libcnb::layer::LayerRef;
use libcnb::read_toml_file;
use libcnb::{build::BuildContext, layer::UncachedLayerDefinition};
use libherokubuildpack::log::log_info;
use libherokubuildpack::toml::toml_select_value;
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

    let project_toml_path = context.app_dir.join("project.toml");

    let heroku_config = if project_toml_path.is_file() {
        let project_toml = read_toml_file::<toml::Value>(project_toml_path)
            .map_err(StaticWebServerBuildpackError::CannotReadProjectToml)?;

        toml_select_value(vec!["com", "heroku", "static-web-server"], &project_toml).map_or(
            Ok(HerokuWebServerConfig::default()),
            |table| {
                table
                    .clone()
                    .try_into()
                    .map_err(StaticWebServerBuildpackError::CannotParseHerokuWebServerConfiguration)
            },
        )?
    } else {
        HerokuWebServerConfig::default()
    };

    let build_command_opt = heroku_config.build.clone();

    let caddy_config = CaddyConfig::try_from(heroku_config)?;

    let caddy_config_json =
        serde_json::to_string(&caddy_config).map_err(StaticWebServerBuildpackError::Json)?;

    let config_path = configuration_layer.path().join("caddy.json");
    fs::write(config_path, caddy_config_json)
        .map_err(StaticWebServerBuildpackError::CannotWriteCaddyConfiguration)?;

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
