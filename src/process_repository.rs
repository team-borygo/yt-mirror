use std::{path::PathBuf, str::from_utf8};

use anyhow::Result;
use rusqlite::{types::FromSql, Connection, ToSql};

use crate::types::{Process, ProcessState};

pub struct ProcessRepository {
    connection: Connection,
}

impl ToSql for ProcessState {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match self {
            ProcessState::Failed => Ok("failed".to_string().into()),
            ProcessState::Pending => Ok("pending".to_string().into()),
            ProcessState::Finished => Ok("finished".to_string().into()),
            ProcessState::Skipped => Ok("skipped".to_string().into()),
        }
    }
}

impl FromSql for ProcessState {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Text(text) => {
                let text = from_utf8(text).expect("Process state is invalid UTF8");
                match text {
                    "pending" => Ok(ProcessState::Pending),
                    "failed" => Ok(ProcessState::Failed),
                    "finished" => Ok(ProcessState::Finished),
                    "skipped" => Ok(ProcessState::Skipped),
                    _ => panic!("Unknown state value for ProcessState"),
                }
            }
            _ => panic!("Unknown column type for ProcessState"),
        }
    }
}

impl ProcessRepository {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let connection = Connection::open(db_path)?;

        connection.execute(
            "CREATE TABLE IF NOT EXISTS process (
                youtubeId TEXT NOT NULL PRIMARY KEY,
                state TEXT NOT NULL,
                errorMessage TEXT
              )",
            (),
        )?;

        Ok(ProcessRepository { connection })
    }

    pub fn get_by_state(&self, state: ProcessState) -> Result<Vec<Process>> {
        let mut stmt = self
            .connection
            .prepare("SELECT youtubeId, state, errorMessage FROM process WHERE state = (?1)")?;

        let iter = stmt.query_map([state], |row| {
            Ok(Process {
                youtube_id: row.get(0)?,
                state: row.get(1)?,
                error: row.get(2)?,
            })
        })?;

        Ok(iter.map(|p| p.unwrap()).collect())
    }

    pub fn finish(&self, id: &str) -> () {
        self.connection
            .execute(
                "UPDATE process SET state = (?1) WHERE youtubeId = (?2)",
                (ProcessState::Finished, id.clone()),
            )
            .expect("Marking process as failed was not successful");
    }

    pub fn fail(&self, id: &str, error: &str) -> () {
        self.connection
            .execute(
                "UPDATE process SET state = (?1), errorMessage = (?2) WHERE youtubeId = (?3)",
                (ProcessState::Failed, error.clone(), id.clone()),
            )
            .expect("Marking process as failed was not successful");
    }

    pub fn skip(&self, id: &str) -> () {
        self.connection
            .execute(
                "UPDATE process SET state = (?1) WHERE youtubeId = (?2)",
                (ProcessState::Skipped, id.clone()),
            )
            .expect("Marking process as skipped was not successful");
    }

    pub fn save_many(&mut self, processes: &Vec<Process>) -> Result<()> {
        let tx = self.connection.transaction()?;

        for process in processes {
            tx.execute(
                "INSERT OR IGNORE INTO process (youtubeId, state, errorMessage) VALUES (?1, ?2, ?3)",
                (process.youtube_id.clone(), process.state.clone(), process.error.clone())
            )?;
        }

        tx.commit()?;

        Ok(())
    }
}
