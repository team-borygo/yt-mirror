use std::{io::Write, path::PathBuf};

use anyhow::Result;

use super::app_config::AppConfig;

pub struct Config {
    config_file: PathBuf,
    config_directory: PathBuf,
    data_directory: PathBuf,
    app_config: AppConfig,
}

impl Config {
    pub fn new_from_directory() -> Result<Config> {
        if cfg!(target_os = "windows") {
            todo!("Windows is not supported")
        } else {
            let config_directory_root =
                std::env::var("XDG_CONFIG_HOME").unwrap_or("~/.config".to_string());
            let data_directory_root =
                std::env::var("XDG_DATA_HOME").unwrap_or("~/.local/share".to_string());

            let config_directory = PathBuf::from(config_directory_root).join("yt-mirror");
            let data_directory = PathBuf::from(data_directory_root).join("yt-mirror");
            let config_file = config_directory.join("config.toml");

            ensure_dir(&data_directory)?;
            ensure_dir(&config_directory)?;

            let app_config: AppConfig = {
                let file_content = ensure_file(
                    &config_file,
                    toml::to_string_pretty(&AppConfig::new_default()).unwrap(),
                )?;

                toml::from_str(&file_content)?
            };

            Ok(Config {
                config_file,
                config_directory,
                data_directory,
                app_config,
            })
        }
    }
}

fn ensure_dir(dir: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(dir)?;

    Ok(())
}

fn ensure_file(file_path: &PathBuf, default: String) -> Result<String> {
    if !file_path.exists() {
        let mut file = std::fs::File::create(file_path)?;
        file.write_all(&default.as_bytes());
        Ok(default)
    } else {
        Ok(std::fs::read_to_string(file_path)?)
    }
}
