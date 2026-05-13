use crate::error::AppError;
use crate::models::app_data_dir;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
pub struct HistoryEntry {
    pub ts: String,
    pub text: String,
    pub model: String,
    pub post_processed: bool,
}

fn history_path() -> PathBuf {
    app_data_dir().join("history.jsonl")
}

pub fn append(entry: &HistoryEntry) -> Result<(), AppError> {
    let p = history_path();
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&p)?;
    writeln!(
        f,
        "{}",
        serde_json::to_string(entry).map_err(|e| AppError::Internal(e.to_string()))?
    )?;
    Ok(())
}

pub fn read_all() -> Result<Vec<HistoryEntry>, AppError> {
    let p = history_path();
    if !p.exists() {
        return Ok(vec![]);
    }
    let f = std::fs::File::open(&p)?;
    let mut out = Vec::new();
    for line in BufReader::new(f).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(e) = serde_json::from_str(&line) {
            out.push(e);
        }
    }
    Ok(out)
}

pub fn delete_by_ts(ts: &str) -> Result<(), AppError> {
    let all: Vec<_> = read_all()?.into_iter().filter(|e| e.ts != ts).collect();
    let p = history_path();
    let mut f = std::fs::File::create(&p)?;
    for e in all {
        writeln!(
            f,
            "{}",
            serde_json::to_string(&e).map_err(|e| AppError::Internal(e.to_string()))?
        )?;
    }
    Ok(())
}

pub fn clear() -> Result<(), AppError> {
    let p = history_path();
    if p.exists() {
        std::fs::remove_file(p)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let e = HistoryEntry {
            ts: "2026-01-01T00:00:00Z".into(),
            text: "hi".into(),
            model: "x".into(),
            post_processed: false,
        };
        let _ = clear();
        append(&e).unwrap();
        let all = read_all().unwrap();
        assert!(all.iter().any(|x| x.text == "hi"));
        delete_by_ts("2026-01-01T00:00:00Z").unwrap();
        assert!(read_all()
            .unwrap()
            .iter()
            .all(|x| x.ts != "2026-01-01T00:00:00Z"));
    }
}
