use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

use chrono::offset::Local;
use chrono::{DateTime, NaiveDateTime, TimeZone};
use serde::de;
use serde::Deserialize;

use crate::opt::Opt;

#[derive(Debug, Deserialize)]
pub struct Task {
    pub id: usize,
    pub description: String,
    pub uuid: String,
    pub tags: Option<Vec<String>>,
    pub wait: Option<ParsableDateTime>,
}

impl Task {
    pub fn has_note(&self, opt: &Opt) -> io::Result<bool> {
        // a lot of editors will keep an "empty" line at the top of a file, so a naive 'byte size
        // == 0' check won't cut it.
        //
        // because we expect notes to be VERY small (on the order of KB at most), we can just scan
        // to see if there's any non-whitespace.
        //
        // NOTE: if perf becomes an issue, this will become a good place to refactor
        let file = match File::open(self.path(opt)) {
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

    pub fn has_tag(&self) -> bool {
        match &self.tags {
            None => false,
            Some(tags) => {
                for tag in tags.iter() {
                    if tag == "taskn" {
                        return true;
                    }
                }
                false
            }
        }
    }

    pub fn path(&self, opt: &Opt) -> PathBuf {
        PathBuf::new()
            .join(&opt.root_dir)
            .join(&self.uuid)
            .with_extension(&opt.file_format)
    }
}

#[derive(Debug)]
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
