/// Provides a high-level wrapper around the eventkit-sys crate defined in this repository.
/// This module allows one to read, create, and update reminders on macOS.
/// Because this module depends on eventkit-sys, do not expect it to compile on non-macOS systems.
use std::ffi::{c_void, CString};

use chrono::{DateTime, Local};
use objc::runtime::Object;

#[link(name = "EventKit", kind = "framework")]
extern "C" {}

#[derive(Debug)]
enum EKError {}

type EKResult<T> = Result<T, EKError>;

struct EventStore {
    ek_event_store: *mut Object,
}

impl EventStore {
    pub fn new() -> Self {
        let mut ek_event_store: *mut Object;
        unsafe {
            let cls = class!(EKEventStore);
            ek_event_store = msg_send![cls, alloc];
            ek_event_store = msg_send![ek_event_store, init];
        }
        Self { ek_event_store }
    }
}

impl Drop for EventStore {
    fn drop(&mut self) {
        unsafe {
            let _: c_void = msg_send![self.ek_event_store, dealloc];
        }
    }
}

struct Reminder {
    pub title: String,
    pub notes: String,
    pub time: DateTime<Local>,
}

unsafe fn to_ek_reminder(reminder: Reminder) -> *mut Object {
    let cls = class!(EKReminder);
    let ek_reminder: *mut Object = msg_send![cls, alloc];

    let ns_title = to_ns_string(reminder.title);
    let _: c_void = msg_send![ek_reminder, setTitle: ns_title];

    let ns_notes = to_ns_string(reminder.notes);
    let _: c_void = msg_send![ek_reminder, setNotes: ns_notes];

    ek_reminder
}

unsafe fn to_ns_string<S: AsRef<str>>(s: S) -> *mut Object {
    let c_string = CString::new(s.as_ref());
    let cls = class!(NSString);
    let ns_string = msg_send![cls, stringWithCString: c_string];
    ns_string
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeZone};

    use super::*;

    #[test]
    fn test_event_store_new() {
        let _ = EventStore::new();
    }

    #[test]
    fn test_to_ns_string() {
        let ns_string: *mut Object;
        unsafe {
            ns_string = to_ns_string("hello world");
        }
    }

    #[test]
    fn test_to_ek_reminder() {
        let event_store = EventStore::new();
        let reminder = Reminder {
            title: "a title".to_string(),
            notes: "a notes".to_string(),
            time: Local.from_utc_datetime(&NaiveDate::from_ymd(2021, 5, 01).and_hms(12, 0, 0)),
        };

        let ek_reminder;
        unsafe {
            ek_reminder = to_ek_reminder(reminder);
            let _: c_void = msg_send![ek_reminder, dealloc];
        }
    }
}
