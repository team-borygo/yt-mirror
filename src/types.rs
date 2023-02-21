#[derive(Debug)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
}

#[derive(Clone)]
pub enum ProcessState {
    Pending,
    Failed,
    Finished,
    Skipped
}

pub struct Process {
    pub youtube_id: String,
    pub state: ProcessState,
    pub error: Option<String>,
}