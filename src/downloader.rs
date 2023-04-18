use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use std::{path::PathBuf, process::Command};

#[derive(Debug, Clone)]
pub enum DownloadResult {
    DownloadSkipped {
        downloader_id: String,
        youtube_id: String,
    },
    DownloadFailed {
        downloader_id: String,
        youtube_id: String,
        error_message: String,
    },
    DownloadFinished {
        downloader_id: String,
        youtube_id: String,
    },
}

#[derive(Debug, Clone)]
pub enum DownloaderState {
    Downloading {
        downloader_id: String,
        youtube_id: String,
    },
    Finished {
        downloader_id: String,
    },
    Waiting {
        downloader_id: String,
    },
    Crashed {
        downloader_id: String,
    },
}

#[derive(Debug)]
pub enum DownloaderMessage {
    Result(DownloadResult),
    State(DownloaderState),
}

pub struct Downloader {
    pub id: String,
    work_channel: Receiver<String>,
    message_channel: Sender<DownloaderMessage>,
    target: PathBuf,
    tmp: PathBuf,
    filter: Option<String>,
}

impl Downloader {
    pub fn new(
        id: String,
        work_channel: Receiver<String>,
        message_channel: Sender<DownloaderMessage>,
        target: PathBuf,
        tmp: PathBuf,
        filter: Option<String>,
    ) -> Self {
        Downloader {
            id,
            work_channel,
            message_channel,
            target,
            tmp,
            filter,
        }
    }

    pub fn start(&self) -> () {
        self.message_channel
            .send(DownloaderMessage::State(DownloaderState::Waiting {
                downloader_id: self.id.clone(),
            }))
            .expect("Cannot send downloader state to message channel");

        while let Ok(youtube_id) = self.work_channel.recv() {
            self.message_channel
                .send(DownloaderMessage::State(DownloaderState::Downloading {
                    downloader_id: self.id.clone(),
                    youtube_id: youtube_id.clone(),
                }))
                .expect("Cannot send downloader state to message channel");

            let result = self.download_yt(youtube_id, &self.target, &self.tmp, &self.filter);

            match result {
                Ok(result) => {
                    self.message_channel
                        .send(DownloaderMessage::Result(result))
                        .expect("Cannot send download result to message channel");
                }
                Err(_) => {
                    self.message_channel
                        .send(DownloaderMessage::State(DownloaderState::Crashed {
                            downloader_id: self.id.clone(),
                        }))
                        .expect("Cannot send downloader state to message channel");
                }
            }
        }

        self.message_channel
            .send(DownloaderMessage::State(DownloaderState::Finished {
                downloader_id: self.id.clone(),
            }))
            .expect("Cannot send downloader state to message channel");
    }

    pub fn download_yt(
        &self,
        youtube_id: String,
        target_dir: &PathBuf,
        tmp_dir: &PathBuf,
        match_filter: &Option<String>,
    ) -> Result<DownloadResult> {
        let output = {
            if cfg!(target_os = "windows") {
                todo!("Windows is not supported")
            } else {
                let mut args = vec![
                    "-x".to_string(),
                    "-o".to_string(),
                    format!("{}/%(title)s.%(ext)s", tmp_dir.display()),
                    "--no-warnings".to_string(),
                    "--exec".to_string(),
                    format!("mv {{}} {}/", target_dir.display()),
                ];

                if let Some(filter) = match_filter {
                    args.push("--match-filter".to_string());
                    args.push(filter.to_string());
                }

                args.push("--".to_string());
                args.push(youtube_id.clone());

                Command::new("yt-dlp").args(args).output()?
            }
        };

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;
        let is_skipped = stdout.starts_with("skipping ..");

        if output.status.success() {
            if is_skipped {
                Ok(DownloadResult::DownloadSkipped {
                    youtube_id,
                    downloader_id: self.id.clone(),
                })
            } else {
                Ok(DownloadResult::DownloadFinished {
                    youtube_id,
                    downloader_id: self.id.clone(),
                })
            }
        } else {
            Ok(DownloadResult::DownloadFailed {
                youtube_id,
                error_message: stderr,
                downloader_id: self.id.clone(),
            })
        }
    }
}
