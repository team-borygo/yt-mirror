use std::{process::{Command}};

use anyhow::{Result};
use crossbeam_channel::{Receiver, Sender};
use nanoid::nanoid;

#[derive(Debug)]
pub enum DownloadStatus {
    DownloadSkipped { youtube_id: String },
    DownloadFailed { youtube_id: String, error_message: String },
    DownloadFinished { youtube_id: String },
}

#[derive(Debug)]
pub struct DownloaderResult {
    pub actor_id: String,
    pub status: DownloadStatus,
}

pub fn download_yt(
    youtube_id: String,
    target_dir: &str,
    tmp_dir: &str,
    match_filter: &Option<String>,
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
                args.push(filter.to_string());
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

pub struct Downloader {
    id: String,
    work_channel: Receiver<String>,
    result_channel: Sender<DownloaderResult>,
    target: String,
    tmp: String,
    filter: Option<String>
}

impl Downloader {
    pub fn new(
        work_channel: Receiver<String>,
        result_channel: Sender<DownloaderResult>,
        target: String,
        tmp: String,
        filter: Option<String>
    ) -> Self {
        Downloader {
            id: nanoid!(5),
            work_channel,
            result_channel,
            target,
            tmp,
            filter,
        }
    }

    pub fn start(&self) -> () {
        while let Ok(youtube_id) = self.work_channel.recv() {
            let result = download_yt(
                youtube_id,
                &self.target,
                &self.tmp,
                &self.filter
            );
            
            match result {
                Ok(status) => {
                    let result = DownloaderResult {
                        actor_id: self.id.to_string(),
                        status,
                    };

                    self.result_channel.send(result).expect(
                        "Cannot send download result to result channel"
                    );
                }
                Err(_) => {
                    todo!()
                }
            }
        }
    }
}