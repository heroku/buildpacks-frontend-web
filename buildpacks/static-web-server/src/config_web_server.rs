use crate::caddy_config::CaddyConfig;
use crate::heroku_web_server_config::{
    HerokuWebServerConfig, RuntimeConfig, DEFAULT_DOC_INDEX, DEFAULT_DOC_ROOT,
};
use crate::{StaticWebServerBuildpack, StaticWebServerBuildpackError, BUILD_PLAN_ID};
use libcnb::additional_buildpack_binary_path;
use libcnb::data::layer_name;
use libcnb::layer::LayerRef;
use libcnb::layer_env::{ModificationBehavior, Scope};
use libcnb::{build::BuildContext, layer::UncachedLayerDefinition};
use libherokubuildpack::log::log_info;
use static_web_server_utils::read_project_config;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use toml::Table;

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

    let build_plan_config = generate_build_plan_config(context);
    let project_config = read_project_config(context.app_dir.as_ref())
        .map_err(StaticWebServerBuildpackError::CannotReadProjectToml)?;

    let heroku_config =
        generate_config_with_inheritance(project_config.as_ref(), &build_plan_config)?;

    let build_command_opt = heroku_config.build.clone();
    let runtime_config_opt = heroku_config.runtime_config.clone();

    // Resolve web root and index doc
    let doc_root_path = heroku_config
        .root
        .clone()
        .unwrap_or(PathBuf::from(DEFAULT_DOC_ROOT));
    let doc_root = doc_root_path.to_string_lossy();
    let doc_index = heroku_config
        .index
        .clone()
        .unwrap_or(DEFAULT_DOC_INDEX.to_string());
    let default_doc_path = format!("{doc_root}/{doc_index}");
    let default_doc_string = if doc_root.is_empty() {
        doc_index.as_str()
    } else {
        default_doc_path.as_str()
    };
    let default_doc_path = Path::new(default_doc_string);

    // Transform web server config to Caddy native JSON config
    let caddy_config = CaddyConfig::try_from(heroku_config)?;
    let caddy_config_json =
        serde_json::to_string(&caddy_config).map_err(StaticWebServerBuildpackError::Json)?;
    let config_path = configuration_layer.path().join("caddy.json");
    fs::write(config_path, caddy_config_json)
        .map_err(StaticWebServerBuildpackError::CannotWriteCaddyConfiguration)?;

    // Execute the optional build command
    if let Some(build_command) = build_command_opt {
        log_info(format!("Executing build command: {build_command:#?}"));
        let mut cmd = Command::new(build_command.command);
        if let Some(args) = build_command.args {
            cmd.args(args);
        }

        cmd.stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map_err(StaticWebServerBuildpackError::BuildCommandFailed)?;
    }

    // Set-up runtime configuration; defaults to enabled
    if runtime_config_opt
        .unwrap_or(RuntimeConfig { enabled: None })
        .enabled
        .unwrap_or(true)
    {
        log_info("Installing runtime configuration process…");
        let web_exec_destination = configuration_layer.path().join("exec.d/web");
        let exec_path = web_exec_destination.join("env-as-html-data");
        log_info(format!("  {}", exec_path.display()));
        fs::create_dir_all(&web_exec_destination)
            .map_err(StaticWebServerBuildpackError::CannotCreatWebExecD)?;
        fs::copy(
            additional_buildpack_binary_path!("env-as-html-data"),
            exec_path,
        )
        .map_err(StaticWebServerBuildpackError::CannotInstallEnvAsHtmlData)?;

        // Set env-to-html-data param as env variable
        let mut configuration_layer_env = configuration_layer.read_env()?;
        configuration_layer_env.insert(
            Scope::Process("web".to_string()),
            ModificationBehavior::Override,
            "ENV_AS_HTML_DATA_TARGET_FILES",
            default_doc_path,
        );
        configuration_layer.write_env(configuration_layer_env)?;
    } else {
        log_info("Runtime configuration is not enabled.");
    }

    Ok(configuration_layer)
}

// Load a table of Build Plan [requires.metadata] from context.
// When a key is defined multiple times,
// * for tables: insert the new row value to the existing table
// * for other value types: the values overwrite, so the last one defined wins
fn generate_build_plan_config(
    context: &BuildContext<StaticWebServerBuildpack>,
) -> toml::map::Map<String, toml::Value> {
    let mut build_plan_config = Table::new();
    context.buildpack_plan.entries.iter().for_each(|e| {
        if e.name == BUILD_PLAN_ID {
            e.metadata.iter().for_each(|(k, v)| {
                if let Some(new_values) = v.as_table() {
                    if let Some(existing_values) =
                        build_plan_config.get(k).and_then(|ev| ev.as_table())
                    {
                        let mut all_values = existing_values.clone();
                        new_values.into_iter().for_each(|(nk, nv)| {
                            all_values.insert(nk.clone(), nv.clone());
                        });
                        build_plan_config.insert(k.to_owned(), all_values.into());
                    } else {
                        build_plan_config.insert(k.to_owned(), v.to_owned());
                    }
                } else {
                    build_plan_config.insert(k.to_owned(), v.to_owned());
                }
            });
        }
    });
    build_plan_config
}

fn generate_config_with_inheritance(
    project_config: Option<&toml::Value>,
    config_to_inherit: &toml::map::Map<String, toml::Value>,
) -> Result<HerokuWebServerConfig, libcnb::Error<StaticWebServerBuildpackError>> {
    // Default config is from the Build Plan metadata or empty.
    let default_config: HerokuWebServerConfig = config_to_inherit
        .clone()
        .try_into()
        .map_err(StaticWebServerBuildpackError::CannotParseHerokuWebServerConfiguration)?;

    let heroku_config: HerokuWebServerConfig =
        project_config.map_or(Ok(default_config), |table| {
            let mut config_from_project: toml::Table = table.clone().try_into().unwrap_or_default();

            config_to_inherit.iter().for_each(|(bpk, bpv)| {
                if !config_from_project.contains_key(bpk) {
                    config_from_project.insert(bpk.to_owned(), bpv.to_owned());
                }
            });
            config_from_project
                .try_into()
                .map_err(StaticWebServerBuildpackError::CannotParseHerokuWebServerConfiguration)
        })?;

    Ok(heroku_config)
}

#[cfg(test)]
mod tests {
    use libcnb::{
        build::BuildContext,
        data::{
            buildpack::{Buildpack, BuildpackApi, BuildpackVersion, ComponentBuildpackDescriptor},
            buildpack_id,
            buildpack_plan::{BuildpackPlan, Entry},
        },
        generic::GenericPlatform,
        Env, Target,
    };
    use std::{collections::HashSet, path::PathBuf};
    use toml::toml;

    use crate::{
        config_web_server::{generate_build_plan_config, generate_config_with_inheritance},
        StaticWebServerBuildpack, BUILD_PLAN_ID,
    };

    #[test]
    fn generate_build_plan_config_from_one_entry() {
        let test_build_plan = vec![Entry {
            name: BUILD_PLAN_ID.to_string(),
            metadata: toml! {
                root = "testY"

                [headers]
                X-Server = "testX"
            },
        }];
        let test_context = create_test_context(test_build_plan);
        let result = generate_build_plan_config(&test_context);

        let result_root = result
            .get("root")
            .expect("should contain the property: root");
        assert_eq!(result_root, &toml::Value::String("testY".to_string()));

        let result_headers = result.get("headers").expect("should contain headers");
        let result_table = result_headers.as_table().expect("should contain atable");
        assert_eq!(
            result_table.get("X-Server"),
            Some(&toml::Value::String("testX".to_string()))
        );
    }

    #[test]
    fn generate_build_plan_config_collects_headers_from_entries() {
        let test_build_plan = vec![
            Entry {
                name: BUILD_PLAN_ID.to_string(),
                metadata: toml! {
                    [headers]
                    X-Serve-1 = "test1"
                },
            },
            Entry {
                name: BUILD_PLAN_ID.to_string(),
                metadata: toml! {
                    [headers]
                    X-Serve-2 = "test2"
                    X-Serve-3 = "test3"
                },
            },
            Entry {
                name: BUILD_PLAN_ID.to_string(),
                metadata: toml! {
                    [headers]
                    X-Serve-4 = "test4"
                },
            },
        ];
        let test_context = create_test_context(test_build_plan);
        let result = generate_build_plan_config(&test_context);

        let result_headers = result.get("headers").expect("should contain headers");
        assert_eq!(
            result_headers.get("X-Serve-1"),
            Some(&toml::Value::String("test1".to_string()))
        );
        assert_eq!(
            result_headers.get("X-Serve-2"),
            Some(&toml::Value::String("test2".to_string()))
        );
        assert_eq!(
            result_headers.get("X-Serve-3"),
            Some(&toml::Value::String("test3".to_string()))
        );
        assert_eq!(
            result_headers.get("X-Serve-4"),
            Some(&toml::Value::String("test4".to_string()))
        );
    }

    #[test]
    fn generate_build_plan_config_captures_last_root_from_entries() {
        let test_build_plan = vec![
            Entry {
                name: BUILD_PLAN_ID.to_string(),
                metadata: toml! {
                    root = "test1"
                },
            },
            Entry {
                name: BUILD_PLAN_ID.to_string(),
                metadata: toml! {
                    root = "test2"
                },
            },
        ];
        let test_context = create_test_context(test_build_plan);
        let result = generate_build_plan_config(&test_context);

        let result_root = result
            .get("root")
            .expect("should contain the property: root");
        assert_eq!(result_root, &toml::Value::String("test2".to_string()));
    }

    #[test]
    fn generate_build_plan_config_empty() {
        let test_build_plan = vec![];
        let test_context = create_test_context(test_build_plan);
        let result = generate_build_plan_config(&test_context);
        assert!(result.is_empty());
    }

    #[test]
    fn generate_config_default() {
        let inherit_config = toml::Table::new();

        let parsed_config = generate_config_with_inheritance(None, &inherit_config).unwrap();
        assert_eq!(parsed_config.build, None);
        assert_eq!(parsed_config.root, None);
        assert_eq!(parsed_config.index, None);
        assert_eq!(parsed_config.headers, None);
    }

    #[test]
    fn generate_config_with_project_toml() {
        let project_config: toml::Value = toml! {
            root = "files/web"
        }
        .into();
        let inherit_config = toml::Table::new();

        let parsed_config =
            generate_config_with_inheritance(Some(&project_config), &inherit_config).unwrap();
        assert_eq!(parsed_config.build, None);
        assert_eq!(parsed_config.root, Some(PathBuf::from("files/web")));
        assert_eq!(parsed_config.index, None);
        assert_eq!(parsed_config.headers, None);
    }

    #[test]
    fn generate_config_with_build_plan() {
        let mut inherit_config = toml::Table::new();
        inherit_config.insert("root".to_string(), "www".to_string().into());

        let parsed_config = generate_config_with_inheritance(None, &inherit_config).unwrap();
        assert_eq!(parsed_config.build, None);
        assert_eq!(parsed_config.root, Some(PathBuf::from("www")));
        assert_eq!(parsed_config.index, None);
        assert_eq!(parsed_config.headers, None);
    }

    #[test]
    fn generate_config_with_project_precedence() {
        let project_config: toml::Value = toml! {
            root = "value/with/precedence"
        }
        .into();
        let mut inherit_config = toml::Table::new();
        inherit_config.insert(
            "root".to_string(),
            "value/should/be/overriden".to_string().into(),
        );
        inherit_config.insert("index".to_string(), "main.html".to_string().into());

        let parsed_config =
            generate_config_with_inheritance(Some(&project_config), &inherit_config).unwrap();
        assert_eq!(parsed_config.build, None);
        assert_eq!(
            parsed_config.root,
            Some(PathBuf::from("value/with/precedence"))
        );
        assert_eq!(parsed_config.index, Some(String::from("main.html")));
        assert_eq!(parsed_config.headers, None);
    }

    fn create_test_context(build_plan: Vec<Entry>) -> BuildContext<StaticWebServerBuildpack> {
        let test_context: BuildContext<StaticWebServerBuildpack> = BuildContext {
            layers_dir: PathBuf::new(),
            app_dir: PathBuf::new(),
            buildpack_dir: PathBuf::new(),
            target: Target {
                os: "test".to_string(),
                arch: "test".to_string(),
                arch_variant: None,
                distro_name: "test".to_string(),
                distro_version: "test".to_string(),
            },
            platform: GenericPlatform::new(<Env as std::default::Default>::default()),
            buildpack_plan: BuildpackPlan {
                entries: build_plan,
            },
            buildpack_descriptor: ComponentBuildpackDescriptor {
                api: BuildpackApi { major: 0, minor: 0 },
                buildpack: Buildpack {
                    id: buildpack_id!("heroku/test"),
                    name: None,
                    version: BuildpackVersion::new(0, 0, 0),
                    homepage: None,
                    clear_env: false,
                    description: None,
                    keywords: vec![],
                    licenses: vec![],
                    sbom_formats: HashSet::new(),
                },
                stacks: vec![],
                targets: vec![],
                metadata: None,
            },
            store: None,
        };
        test_context
    }
}
