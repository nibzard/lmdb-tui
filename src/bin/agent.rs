use std::io::{self, BufRead, Write};
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use lmdb_tui::db::env::{list_databases, open_env};

#[derive(Serialize, Deserialize)]
enum Request {
    ListDbs { path: String, read_only: bool },
}

#[derive(Serialize, Deserialize)]
enum Response {
    Dbs(Vec<String>),
    Error(String),
}

fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let req: Request = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = Response::Error(e.to_string());
                serde_json::to_writer(&mut stdout, &resp)?;
                stdout.write_all(b"\n")?;
                stdout.flush()?;
                continue;
            }
        };
        let resp = handle(req);
        match resp {
            Ok(r) => serde_json::to_writer(&mut stdout, &r)?,
            Err(e) => serde_json::to_writer(&mut stdout, &Response::Error(e.to_string()))?,
        }
        stdout.write_all(b"\n")?;
        stdout.flush()?;
    }
    Ok(())
}

fn handle(req: Request) -> Result<Response> {
    match req {
        Request::ListDbs { path, read_only } => {
            let env = open_env(Path::new(&path), read_only)?;
            let names = list_databases(&env)?;
            Ok(Response::Dbs(names))
        }
    }
}
