use std::fmt;
use std::fs::File;
#[cfg(feature = "eventkit")]
use std::fs::OpenOptions;
use std::io::{self, Read};
#[cfg(feature = "eventkit")]
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::Command;
use std::str;

use chrono::offset::Local;
use chrono::{DateTime, NaiveDateTime, TimeZone};
use serde::de;
use serde::Deserialize;

#[cfg(feature = "eventkit")]
use shellexpand::tilde;

use crate::opt::Opt;

#[derive(Debug, Deserialize)]
pub struct Task {
    pub id: usize,
    pub description: String,
    pub uuid: String,
    pub status: String,
    pub estimate: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub wait: Option<ParsableDateTime>,
    pub taskn_reminder_uuid: Option<String>,
}

impl Task {
    /// Saves anything stored inside this Task to taskwarrior.
    pub fn save(&self) -> io::Result<()> {
        let mut command = Command::new("task");
        command
            .arg(&self.uuid)
            .arg("modify")
            .arg(&self.description)
            .arg(format!("status:{}", self.status));

        if let Some(estimate) = self.estimate {
            command.arg(format!("estimate:{}", estimate));
        } else {
            command.arg("estimate:");
        }

        if let Some(_wait) = &self.wait {
            // TODO: update wait when it exists
            // command.arg(format!("wait:{}", wait));
        } else {
            command.arg("wait:");
        }

        if let Some(taskn_reminder_uuid) = &self.taskn_reminder_uuid {
            command.arg(format!("taskn_reminder_uuid:{}", taskn_reminder_uuid));
        } else {
            command.arg("taskn_reminder_uuid:");
        }

        let _ = command.output()?;
        Ok(())
    }

    /// Loads the contents of the note associated with a particular Task. Note that this requires
    /// the [Opt] parameter because it determines where the tasks are saved.
    pub fn load_contents(&self, opt: &Opt) -> io::Result<String> {
        let path = PathBuf::new()
            .join(&opt.root_dir)
            .join(&self.uuid)
            .with_extension(&opt.file_format);
        match File::open(path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok("".to_string()),
            Err(e) => Err(e),
            Ok(mut file) => {
                let mut buffer = String::new();
                file.read_to_string(&mut buffer)?;
                Ok(buffer)
            }
        }
    }

    pub fn get<'a, S: ToString, I: Iterator<Item = S>>(
        taskwarrior_args: I,
    ) -> io::Result<Vec<Self>> {
        let taskwarrior_args = taskwarrior_args
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let output = Command::new("task")
            .args(taskwarrior_args)
            .arg("export")
            .output()?;

        let output = match String::from_utf8(output.stdout) {
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "taskwarrior output invalid utf8",
                ))
            }
            Ok(output) => output,
        };

        match serde_json::from_str::<Vec<Self>>(&output) {
            // TODO: report error here
            Err(e) => {
                println!("{:?}", e);
                Err(io::Error::new(io::ErrorKind::InvalidData, ""))
            }
            Ok(tasks) => Ok(tasks),
        }
    }

    pub fn set_estimate(&mut self, estimate: Option<i32>) -> io::Result<()> {
        let estimate_arg;
        if let Some(estimate) = estimate {
            estimate_arg = format!("estimate:{}", estimate);
        } else {
            estimate_arg = "estimate:".to_string();
        }

        Command::new("task")
            .arg(&self.uuid)
            .arg("modify")
            .arg(estimate_arg)
            .output()?;

        Ok(())
    }

    #[cfg(feature = "eventkit")]
    /// Defines a user defined attribute (UDA) that stores the UUID of an operating system reminder
    /// onto the taskwarrior task.
    pub fn define_reminder_uda() -> io::Result<()> {
        let conf_line = "uda.taskn_reminder_uuid.type=string";
        let taskrc_path = tilde("~/.taskrc");

        let mut has_reminder_uda = false;
        {
            let taskrc = BufReader::new(File::open(taskrc_path.as_ref())?);
            for line in taskrc.lines() {
                let line = line?;
                if line == conf_line {
                    has_reminder_uda = true;
                    break;
                }
            }
        }

        if !has_reminder_uda {
            let mut taskrc = OpenOptions::new().append(true).open(taskrc_path.as_ref())?;
            writeln!(taskrc, "{}", conf_line)?;
        }

        Ok(())
    }

    /// Determines whether or not the [Task] contains a tag with the provided value.
    pub fn has_tag<S: AsRef<str>>(&self, s: S) -> bool {
        match &self.tags {
            None => false,
            Some(tags) => {
                let s = s.as_ref();
                for tag in tags.into_iter() {
                    if tag == s {
                        return true;
                    }
                }
                false
            }
        }
    }

    #[cfg(feature = "eventkit")]
    pub fn set_reminder_uuid(&mut self, uuid: String) -> io::Result<()> {
        Command::new("task")
            .arg(&self.uuid)
            .arg("modify")
            .arg(format!("taskn_reminder_uuid:{}", uuid))
            .output()?;

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct ParsableDateTime(pub DateTime<Local>);

impl<'de> Deserialize<'de> for ParsableDateTime {
    fn deserialize<D: de::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<ParsableDateTime, D::Error> {
        Ok(ParsableDateTime(
            deserializer.deserialize_str(DateTimeVisitor)?,
        ))
    }
}

struct DateTimeVisitor;

impl<'de> de::Visitor<'de> for DateTimeVisitor {
    type Value = DateTime<Local>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string encoded in %Y%m%dT%H%M%SZ")
    }

    fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
        // this is a little cursed, but for good reason
        // chrono isn't happy parsing a DateTime without an associated timezone
        // so we parse a DateTime first
        // and then we know it's always in UTC so we make a DateTime<Local> from it
        // and finally convert that back into the DateTime, which is what we want
        NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%SZ")
            .map(|naive_date_time| Local.from_utc_datetime(&naive_date_time))
            .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(s), &self))
    }
}
