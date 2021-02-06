use std::env;
use std::fs::metadata;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

use serde::Deserialize;
use serde_json;
use shellexpand;

fn main() -> io::Result<()> {
    let args = env::args().skip(1).collect::<Vec<String>>();
    let output = Command::new("task")
        .args(args)
        .arg("(status:pending or status:waiting)")
        .arg("export")
        .output()?;
    let output = String::from_utf8(output.stdout).unwrap();
    let tasks = serde_json::from_str::<Vec<Task>>(&output).unwrap();

    let root_dir = shellexpand::tilde("~/.taskn").into_owned();
    let status = Command::new("mkdir")
        .arg("-p")
        .arg(&root_dir)
        .output()?
        .status;
    if !status.success() {
        // TODO: handle error
    }

    let editor = if let Ok(editor) = env::var("EDITOR") {
        editor
    } else {
        // TODO: better default? emacs?
        "vi".to_string()
    };
    let status = Command::new(editor)
        .args(
            tasks
                .iter()
                .map(|task| task.path(&root_dir))
                .collect::<Vec<PathBuf>>(),
        )
        .status()?;
    if !status.success() {
        // TODO: handle error
    }

    for task in tasks.iter() {
        let has_note = task.has_note(&root_dir)?;
        let has_tag = task.has_tag();
        let mut status = None;
        // TODO: maaaaybe switch this out with mostly the same implementation that varies by the
        // argument?
        if has_note && !has_tag {
            status = Some(
                Command::new("task")
                    .arg(&task.uuid)
                    .arg("modify")
                    .arg("+taskn")
                    .output()?
                    .status,
            );
        } else if !has_note && has_tag {
            status = Some(
                Command::new("task")
                    .arg(&task.uuid)
                    .arg("modify")
                    .arg("-taskn")
                    .output()?
                    .status,
            );
        }
        if let Some(status) = status {
            if !status.success() {
                // TODO: handle error
            }
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct Task {
    id: usize,
    uuid: String,
    tags: Option<Vec<String>>,
}

impl Task {
    fn has_note<P: AsRef<Path>>(&self, root_dir: P) -> io::Result<bool> {
        // if there's only a newline in a file, then this will return true even though there's
        // effectively not a note
        match metadata(self.path(root_dir)) {
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    return Ok(false);
                } else {
                    return Err(e);
                }
            }
            Ok(metadata) => Ok(metadata.len() > 0),
        }
    }

    fn has_tag(&self) -> bool {
        match &self.tags {
            None => false,
            Some(tags) => {
                for tag in tags.into_iter() {
                    if tag == "taskn" {
                        return true;
                    }
                }
                false
            }
        }
    }

    fn filename(&self) -> PathBuf {
        PathBuf::new().join(&self.uuid).with_extension("md")
    }

    fn path<P: AsRef<Path>>(&self, root_dir: P) -> PathBuf {
        PathBuf::new().join(root_dir).join(self.filename())
    }
}
