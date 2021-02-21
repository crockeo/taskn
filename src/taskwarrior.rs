use std::fmt;

use chrono::offset::Local;
use chrono::{DateTime, NaiveDateTime, TimeZone};
use serde::de;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Task {
    pub id: usize,
    pub description: String,
    pub uuid: String,
    pub tags: Option<Vec<String>>,
    pub wait: Option<ParsableDateTime>,
}

impl Task {
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
