use std::process::{Command};

use anyhow::{Result};

pub enum DownloadStatus {
    DownloadSkipped { youtube_id: String },
    DownloadFailed { youtube_id: String, error_message: String },
    DownloadFinished { youtube_id: String },
}

pub fn download_yt(
    youtube_id: String,
    target_dir: String,
    tmp_dir: String,
    match_filter: Option<String>,
) -> Result<DownloadStatus> {
    let output = {
        if cfg!(target_os = "windows") {
            todo!("Windows is not supported")
        } else {
            let mut args = vec![
                "-x".to_string(),
                "-o".to_string(),
                format!("{}/%(title)s.%(ext)s", tmp_dir),
                "--no-warnings".to_string(),
                "--exec".to_string(),
                format!("mv {{}} {}/", target_dir)
            ];

            if let Some(filter) = match_filter {
                args.push("--match-filter".to_string());
                args.push(filter);
            }

            args.push("--".to_string());
            args.push(youtube_id.clone());

            Command::new("yt-dlp")
                .args(args)
                .output()?
        }
    };

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    let is_skipped = stdout.starts_with("skipping ..");

    if output.status.success() {
        if is_skipped {
            Ok(DownloadStatus::DownloadSkipped { youtube_id })
        } else {
            Ok(DownloadStatus::DownloadFinished { youtube_id })
        }
    } else {
        Ok(DownloadStatus::DownloadFailed { youtube_id, error_message: stderr })
    }
}