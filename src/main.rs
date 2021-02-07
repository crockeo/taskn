use std::env;
use std::fs;
use std::fs::metadata;
use std::io;
use std::path::PathBuf;
use std::process::{exit, Command};
use std::str;

use serde::Deserialize;
use serde_json;
use shellexpand;
use structopt::StructOpt;

fn main() -> io::Result<()> {
    let opt = ProtoOpt::from_args().into_opt();

    let output = String::from_utf8(
        Command::new("task")
            .args(&opt.args)
            .arg("export")
            .output()?
            .stdout,
    )
    .unwrap();
    let tasks = serde_json::from_str::<Vec<Task>>(&output).unwrap();

    if let Err(_) = fs::create_dir_all(&opt.root_dir) {
        eprintln!("Failed to create taskn directory '{}'", &opt.root_dir);
        exit(1)
    }

    let status = Command::new(&opt.editor)
        .args(
            tasks
                .iter()
                .map(|task| task.path(&opt))
                .collect::<Vec<PathBuf>>(),
        )
        .status()?;
    if !status.success() {
        eprintln!("Failed to open editor '{}' ", &opt.editor);
        exit(1)
    }

    for task in tasks.iter() {
        let has_note = task.has_note(&opt)?;
        let has_tag = task.has_tag();

        let action = if has_note && !has_tag {
            Some("+taskn")
        } else if !has_note && has_tag {
            Some("-taskn")
        } else {
            None
        };

        if let Some(action) = action {
            let status = Command::new("task")
                .arg(&task.uuid)
                .arg("modify")
                .arg(action)
                .output()?
                .status;
            if !status.success() {
                eprintln!("Failed to annotate task '{}' with taskn status", task.id);
                exit(1)
            }
        }
    }

    Ok(())
}

#[derive(StructOpt)]
#[structopt(name = "taskn", about = "Taskwarrior task annotation helper")]
struct ProtoOpt {
    /// The editor used to open task notes. If unset, taskn will attempt to use $EDITOR. If $EDITOR
    /// is also unset, taskn will use vi.
    #[structopt(long)]
    editor: Option<String>,

    /// The file format used for task notes.
    #[structopt(long, default_value = "md")]
    file_format: String,

    /// The directory in which task notes are placed. If the directory does not already exist,
    /// taskn will create it.
    #[structopt(long, default_value = "~/.taskn")]
    root_dir: String,

    args: Vec<String>,
}

impl ProtoOpt {
    fn into_opt(self) -> Opt {
        let editor = if let Some(editor) = self.editor {
            editor
        } else if let Ok(editor) = env::var("EDITOR") {
            editor
        } else {
            "vi".to_string()
        };

        let root_dir = shellexpand::tilde(&self.root_dir).to_string();

        Opt {
            editor,
            file_format: self.file_format,
            root_dir,
            args: self.args,
        }
    }
}

struct Opt {
    editor: String,
    file_format: String,
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
    fn has_note(&self, opt: &Opt) -> io::Result<bool> {
        // if there's only a newline in a file, then this will return true even though there's
        // effectively not a note
        match metadata(self.path(opt)) {
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

    fn path(&self, opt: &Opt) -> PathBuf {
        PathBuf::new()
            .join(&opt.root_dir)
            .join(&self.uuid)
            .with_extension(&opt.file_format)
    }
}
