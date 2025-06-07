use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum Request {
    ListDbs { path: String, read_only: bool },
}

#[derive(Serialize, Deserialize)]
enum Response {
    Dbs(Vec<String>),
    Error(String),
}

pub struct RemoteClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl RemoteClient {
    pub fn connect(host: &str) -> Result<Self> {
        let mut cmd = if host == "local" {
            if let Ok(path) = std::env::var("LMDB_TUI_AGENT_PATH") {
                Command::new(path)
            } else {
                Command::new("lmdb-tui-agent")
            }
        } else {
            let mut c = Command::new("ssh");
            c.arg(host).arg("lmdb-tui-agent");
            c
        };
        let mut child = cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("failed to open stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("failed to open stdout"))?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    pub fn list_databases(&mut self, path: &Path, read_only: bool) -> Result<Vec<String>> {
        let req = Request::ListDbs {
            path: path.to_string_lossy().into_owned(),
            read_only,
        };
        serde_json::to_writer(&mut self.stdin, &req)?;
        self.stdin.write_all(b"\n")?;
        self.stdin.flush()?;
        let mut line = String::new();
        self.stdout.read_line(&mut line)?;
        let resp: Response = serde_json::from_str(line.trim_end())?;
        match resp {
            Response::Dbs(names) => Ok(names),
            Response::Error(e) => Err(anyhow!(e)),
        }
    }
}

impl Drop for RemoteClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}
