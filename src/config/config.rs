use home_dir::HomeDirExt;
use std::{io::Write, path::PathBuf};

use anyhow::{anyhow, Result};

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

            let config = Config {
                config_file,
                config_directory,
                data_directory,
                app_config,
            };

            config.validate().and(Ok(config))
        }
    }

    pub fn get_process_path(&self) -> PathBuf {
        self.data_directory.join("processes.sqlite")
    }

    pub fn get_tmp_dir(&self) -> PathBuf {
        let default = PathBuf::from("/tmp");

        self.app_config
            .tmp_dir
            .as_ref()
            .map(|p| p.expand_home().unwrap()) // Not sure how to handle that error
            .unwrap_or(default)
    }

    pub fn get_bookmark_files(&self) -> Vec<PathBuf> {
        self.app_config
            .bookmark_files
            .iter()
            .map(|f| PathBuf::from(f).expand_home().unwrap())
            .collect()
    }

    pub fn get_target_dir(&self) -> PathBuf {
        self.app_config.target_dir.expand_home().unwrap()
    }

    pub fn validate(&self) -> Result<()> {
        let target_dir = self.get_target_dir();
        let tmp_dir = self.get_tmp_dir();
        let bookmark_files = self.get_bookmark_files();

        if !target_dir.exists() {
            return Err(anyhow!(
                "Given target_dir (\"{}\") doesn't exist (config file path: \"{}\")",
                target_dir.display(),
                self.config_file.display()
            ));
        }

        if !tmp_dir.exists() {
            return Err(anyhow!(
                "Given tmp_dir (\"{}\") doesn't exist (config file path: \"{}\")",
                tmp_dir.display(),
                self.config_file.display()
            ));
        }

        if bookmark_files.len() == 0 {
            return Err(anyhow!(
                "Given bookmark_files list is empty (config file path: \"{}\")",
                self.config_file.display()
            ));
        }

        if bookmark_files.iter().any(|f| !f.exists()) {
            return Err(anyhow!(
                "Some of the bookmark_files doesn't exist (config file path: \"{}\")",
                self.config_file.display()
            ));
        }

        Ok(())
    }
}

fn ensure_dir(dir: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(dir)?;

    Ok(())
}

fn ensure_file(file_path: &PathBuf, default: String) -> Result<String> {
    if !file_path.exists() {
        let mut file = std::fs::File::create(file_path)?;
        file.write_all(&default.as_bytes())?;
        Ok(default)
    } else {
        Ok(std::fs::read_to_string(file_path)?)
    }
}

#[cfg(test)]
mod validation {
    // @TODO add dedicated errors

    use std::path::PathBuf;

    use crate::config::app_config::AppConfig;

    use super::Config;

    #[test]
    fn it_should_reject_not_existing_target_dir() -> () {
        let config = Config {
            config_directory: PathBuf::new(),
            config_file: PathBuf::new(),
            data_directory: PathBuf::new(),
            app_config: AppConfig {
                bookmark_files: vec!["/tmp".to_string()],
                tmp_dir: None,
                target_dir: "/foobar".to_string(),
            },
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn it_should_reject_not_existing_tmp_dir() -> () {
        let config = Config {
            config_directory: PathBuf::new(),
            config_file: PathBuf::new(),
            data_directory: PathBuf::new(),
            app_config: AppConfig {
                bookmark_files: vec!["/tmp".to_string()],
                tmp_dir: Some("/foobar".to_string()),
                target_dir: "/tmp".to_string(),
            },
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn it_should_reject_empty_bookmark_files() -> () {
        let config = Config {
            config_directory: PathBuf::new(),
            config_file: PathBuf::new(),
            data_directory: PathBuf::new(),
            app_config: AppConfig {
                bookmark_files: vec![],
                tmp_dir: None,
                target_dir: "/tmp".to_string(),
            },
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn it_should_reject_not_existing_bookmark_files() -> () {
        let config = Config {
            config_directory: PathBuf::new(),
            config_file: PathBuf::new(),
            data_directory: PathBuf::new(),
            app_config: AppConfig {
                bookmark_files: vec!["/tmp".to_string(), "/foobar".to_string()],
                tmp_dir: None,
                target_dir: "/tmp".to_string(),
            },
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn it_should_accept_correct_config() -> () {
        let config = Config {
            config_directory: PathBuf::new(),
            config_file: PathBuf::new(),
            data_directory: PathBuf::new(),
            app_config: AppConfig {
                bookmark_files: vec!["/tmp".to_string()],
                tmp_dir: None,
                target_dir: "/tmp".to_string(),
            },
        };

        assert!(config.validate().is_ok());
    }
}
