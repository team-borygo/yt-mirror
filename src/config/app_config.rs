use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct AppConfig {
    pub(super) bookmark_files: Vec<String>,
    pub(super) target_dir: String,
    pub(super) tmp_dir: Option<String>,
    pub(super) data_dir: Option<String>,
}

impl AppConfig {
    pub fn new_default() -> AppConfig {
        AppConfig {
            bookmark_files: vec![],
            target_dir: "".to_string(),
            data_dir: None,
            tmp_dir: None,
        }
    }
}
