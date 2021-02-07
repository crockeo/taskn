use std::env;
use std::fs::metadata;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

use serde::Deserialize;
use serde_json;
use shellexpand;
use structopt::StructOpt;

fn main() -> io::Result<()> {
    let opt = Opt::from_args();
    let output = Command::new("task").args(opt.args).arg("export").output()?;
    let output = String::from_utf8(output.stdout).unwrap();
    let tasks = serde_json::from_str::<Vec<Task>>(&output).unwrap();

    let root_dir = shellexpand::tilde(&opt.root_dir).into_owned();
    let file_format = opt.file_format;
    let status = Command::new("mkdir")
        .arg("-p")
        .arg(&root_dir)
        .output()?
        .status;
    if !status.success() {
        // TODO: handle error
    }

    let editor = if let Some(editor) = opt.editor {
        editor
    } else if let Ok(editor) = env::var("EDITOR") {
        editor
    } else {
        "vi".to_string()
    };
    let status = Command::new(editor)
        .args(
            tasks
                .iter()
                .map(|task| task.path(&root_dir, &file_format))
                .collect::<Vec<PathBuf>>(),
        )
        .status()?;
    if !status.success() {
        // TODO: handle error
    }

    for task in tasks.iter() {
        let has_note = task.has_note(&root_dir, &file_format)?;
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

#[derive(StructOpt)]
#[structopt(name = "taskn", about = "Taskwarrior task annotation helper")]
struct Opt {
    editor: Option<String>,

    #[structopt(default_value = "md")]
    file_format: String,

    #[structopt(default_value = "~/.taskn")]
    root_dir: String,

    args: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Task {
    id: usize,
    uuid: String,
    tags: Option<Vec<String>>,
}

impl Task {
    fn has_note<P: AsRef<Path>, U: AsRef<Path>>(
        &self,
        root_dir: P,
        file_format: U,
    ) -> io::Result<bool> {
        // if there's only a newline in a file, then this will return true even though there's
        // effectively not a note
        match metadata(self.path(root_dir, file_format)) {
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

    fn path<P: AsRef<Path>, U: AsRef<Path>>(&self, root_dir: P, file_format: U) -> PathBuf {
        PathBuf::new()
            .join(root_dir.as_ref())
            .join(&self.uuid)
            .with_extension(file_format.as_ref())
    }
}
