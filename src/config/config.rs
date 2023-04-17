pub struct Config {
    config_file: PathBuf,
    config_directory: PathBuf,
    data_directory: PathBuf,
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
            ensure_file(&config_file)?;

            Ok(Config {
                config_file,
                config_directory,
                data_directory,
            })
        }
    }
}

fn ensure_dir(dir: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(dir)?;

    Ok(())
}

fn ensure_file(file_path: &PathBuf) -> Result<()> {
    if !file_path.exists() {
        std::fs::File::create(file_path)?;
    }

    Ok(())
}
