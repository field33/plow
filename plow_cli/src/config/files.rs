pub mod credentials;
pub mod workspace_config;
pub mod workspace_manifest;

// pub fn get_registry_url() -> Result<String, CliError> {
//     let config_file_path = camino::Utf8PathBuf::from("./Plow.toml");
//     let config_file_contents =
//         std::fs::read_to_string(&config_file_path).map_err(|_| FailedToReadWorkspaceConfigFile)?;
//     let config_file = toml::from_str::<PlowConfigFile>(&config_file_contents)
//         .map_err(|_| FailedToReadWorkspaceConfigFile)?;
//     Ok(config_file.registry.url.to_owned())
// }
