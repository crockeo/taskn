use std::fs::create_dir_all;
use std::io;
use std::path::PathBuf;
use std::process::{exit, Command};

use crate::opt::Opt;
use crate::Task;

pub fn edit_notes(opt: Opt) -> io::Result<()> {
    let output = String::from_utf8(
        Command::new("task")
            .args(&opt.args)
            .arg("export")
            .output()?
            .stdout,
    )
    .unwrap();
    let tasks = serde_json::from_str::<Vec<Task>>(&output).unwrap();

    if create_dir_all(&opt.root_dir).is_err() {
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
