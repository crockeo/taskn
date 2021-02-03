use std::env;
use std::io;
use std::process::Command;
use std::str;

use serde::Deserialize;
use serde_json;

fn main() -> io::Result<()> {
    let args = env::args().skip(1).collect::<Vec<String>>();
    let output = Command::new("task").arg("export").args(args).output()?;
    let output = String::from_utf8(output.stdout).unwrap();
    let tasks = serde_json::from_str::<Vec<Task>>(&output).unwrap();

    let status = Command::new("mkdir")
        .arg("-p")
        // TODO: expand this tilde so it's not pointing to the verbatim ~ directory
        .arg("~/.taskn")
        .output()?
        .status;
    if !status.success() {
        // TODO: handle error
    }

    // TODO: configure editor (and use something more normal by default?)
    let status = Command::new("nvim")
        .args(
            tasks
                .into_iter()
                .filter(|task| task.status.is_active())
                .map(|task| format!("~/.taskn/{}.md", task.uuid))
                .collect::<Vec<String>>(),
        )
        .status()?;
    if !status.success() {
        // TODO: handle error
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct Task {
    id: usize,
    uuid: String,
    status: Status,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    Pending,
    Waiting,
    Completed,
    Deleted,
}

impl Status {
    fn is_active(&self) -> bool {
        use Status::*;
        match self {
            Pending | Waiting => true,
            _ => false,
        }
    }
}
