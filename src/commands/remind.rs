/// Provides a taskn command that, on macOS, automatically generates reminders for tasks that are
/// set to be +WAITING and +DUE.
///
/// This module is separated into two conceptual parts:
///   1. The OS / program-specific reminder interface
///   2. The general taskwarrior reminder interface
use std::ffi::OsStr;
use std::io;
use std::process::{self, Command};

use chrono::{DateTime, Local};

use crate::taskwarrior::Task;

pub fn set_reminders<R: Reminder, S: AsRef<OsStr>>(taskwarrior_args: Vec<S>) -> io::Result<()> {
    let output = String::from_utf8(
        Command::new("task")
            .args(taskwarrior_args)
            .arg("+WAITING")
            .arg("export")
            .output()?
            .stdout,
    )
    .unwrap();
    let tasks = serde_json::from_str::<Vec<Task>>(&output).unwrap();
    for task in tasks.into_iter() {
        let wait = match task.wait {
            None => continue,
            Some(wait) => wait,
        };
        let has_reminder = task
            .tags
            .map_or(false, |tags| tags.contains(&"reminder".to_string()));
        if !has_reminder {
            R::add_reminder(&task.uuid, &task.description, wait.0)?;
        }
    }

    Ok(())
}

pub trait Reminder {
    fn add_reminder(uuid: &str, title: &str, datetime: DateTime<Local>) -> io::Result<()>;
}

/// Provides an implementation of [Reminder] based on macOS's Reminders app & osascript.
pub struct MacReminder {}

impl MacReminder {
    fn run_osascript(script: &str) -> io::Result<process::Output> {
        Command::new("osascript").arg("-e").arg(script).output()
    }
}

impl Reminder for MacReminder {
    fn add_reminder(uuid: &str, title: &str, datetime: DateTime<Local>) -> io::Result<()> {
        Command::new("task")
            .arg(uuid)
            .arg("modify")
            .arg("+reminder")
            .output()?;

        let osascript = format!("
set datetime to current date
tell datetime to set {{its year, its month, its day}} to words of \"{date}\"
tell datetime to set {{its hours, its minutes}} to words of \"{time}\"
tell app \"Reminders\"
    tell list \"Reminders\" of default account
        make new reminder with properties {{name:\"{title}\", body:\"{uuid}\", remind me date:datetime}}
    end
end", date = datetime.format("%Y-%m-%d"), time = datetime.format("%H:%M"), title = title, uuid = uuid);
        Self::run_osascript(&osascript)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_reminders() -> io::Result<()> {
        set_reminders::<MacReminder, _>(vec!["24"])
    }
}
