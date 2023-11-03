use crate::{commands, config_parsing, project_paths::ParsedProjectPaths};
use anyhow::{Context, Result};

pub async fn run_codegen(project_paths: &ParsedProjectPaths) -> Result<()> {
    let yaml_config = config_parsing::deserialize_config_from_yaml(&project_paths.config)
        .context("Failed deserializing config")?;

    let config =
        config_parsing::config::Config::parse_from_yaml_config(&yaml_config, project_paths)
            .context("Failed parsing config")?;

    commands::codegen::run_codegen(&config, project_paths).await?;
    commands::codegen::run_post_codegen_command_sequence(&project_paths).await?;

    Ok(())
}
