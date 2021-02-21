use std::fs::{create_dir_all, File};
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use std::process::{exit, Command};

use crate::opt::Opt;
use crate::taskwarrior::Task;

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
                .map(|task| task_path(&opt, task))
                .collect::<Vec<PathBuf>>(),
        )
        .status()?;
    if !status.success() {
        eprintln!("Failed to open editor '{}' ", &opt.editor);
        exit(1)
    }

    for task in tasks.iter() {
        let has_note = task_has_note(&opt, task)?;
        let has_tag = task.has_tag("taskn");

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

fn task_has_note(opt: &Opt, task: &Task) -> io::Result<bool> {
    // a lot of editors will keep an "empty" line at the top of a file, so a naive 'byte size
    // == 0' check won't cut it.
    //
    // because we expect notes to be VERY small (on the order of KB at most), we can just scan
    // to see if there's any non-whitespace.
    //
    // NOTE: if perf becomes an issue, this will become a good place to refactor
    let file = match File::open(task_path(opt, task)) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
        Ok(file) => file,
    };
    let reader = BufReader::new(file);
    for line in reader.lines() {
        for c in line?.chars() {
            if !c.is_whitespace() {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn task_path(opt: &Opt, task: &Task) -> PathBuf {
    PathBuf::new()
        .join(&opt.root_dir)
        .join(&task.uuid)
        .with_extension(&opt.file_format)
}
