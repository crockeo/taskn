//! Provides a high-level wrapper around the eventkit-sys crate defined in this repository.
//! This module allows one to read, create, and update reminders on macOS.
//! Because this module depends on eventkit-sys, do not expect it to compile on non-macOS systems.
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

    pub fn save_reminder(&mut self, reminder: Reminder, commit: bool) -> EKResult<bool> {
        let ns_error: *mut Object;
        let saved: bool;
        unsafe {
            ns_error = msg_send![class!(NSError), alloc];
            saved =
                msg_send![self.ek_event_store, saveReminder:reminder commit:commit error:ns_error];
        }

        // TODO: handle the error

        unsafe {
            let _: c_void = msg_send![ns_error, dealloc];
        }

        Ok(saved)
    }
}

impl Drop for EventStore {
    fn drop(&mut self) {
        unsafe {
            let _: c_void = msg_send![self.ek_event_store, release];
        }
    }
}

struct Reminder {
    ek_reminder: *mut Object,
}

impl Reminder {
    fn new<S: AsRef<str>>(title: S, notes: S, time: DateTime<Local>) -> Self {
        let cls = class!(EKReminder);
        let ek_reminder: *mut Object;
        unsafe {
            ek_reminder = msg_send![cls, alloc];

            let ns_title = to_ns_string(title);
            let ns_notes = to_ns_string(notes);
            let _: c_void = msg_send![ek_reminder, setTitle: ns_title];
            let _: c_void = msg_send![ek_reminder, setNotes: ns_notes];

            // TODO: assign a time and make an alarm(?)
        }
        Self { ek_reminder }
    }
}

impl Drop for Reminder {
    fn drop(&mut self) {
        unsafe {
            let ns_title: *mut Object = msg_send![self.ek_reminder, title];
            let _: c_void = msg_send![ns_title, dealloc];

            let ns_notes: *mut Object = msg_send![self.ek_reminder, notes];
            let _: c_void = msg_send![ns_notes, dealloc];

            let _: c_void = msg_send![self.ek_reminder, dealloc];
        }
    }
}

/// Converts a str-like to an
/// [NSString](https://developer.apple.com/documentation/foundation/nsstring?language=objc)
/// returning it as a `*mut Object`. It is the responsibility of the caller to free this string.
///
/// # Arguments
///
/// * `s` - The string we want to convert to an NSString. This can be owned or unowned.
unsafe fn to_ns_string<S: AsRef<str>>(s: S) -> *mut Object {
    // TODO: we're constructing an owned object, c_string, from the ref we receive
    // but we could totally avoid that by just using a CStr instead(?)

    // convert the rust string into a CString ptr
    let c_string = CString::new(s.as_ref()).unwrap().into_raw();

    // turn that UTF8 encoded CString into an NSString
    let cls = class!(NSString);
    let mut ns_string: *mut Object;
    ns_string = msg_send![cls, alloc];
    ns_string = msg_send![ns_string, initWithUTF8String: c_string];

    // resume ownership of the CString to drop it
    let _ = CString::from_raw(c_string);

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
            let _: c_void = msg_send![ns_string, release];
        }
    }

    #[test]
    fn test_reminder_new() {
        let _ = Reminder::new(
            "a title",
            "a notes",
            Local.from_utc_datetime(&NaiveDate::from_ymd(2021, 5, 01).and_hms(12, 0, 0)),
        );
    }

    #[test]
    fn test_save_reminder() -> EKResult<()> {
        let mut event_store = EventStore::new();
        let reminder = Reminder::new(
            "a title",
            "a notes",
            Local.from_utc_datetime(&NaiveDate::from_ymd(2021, 5, 01).and_hms(12, 0, 0)),
        );
        let saved = event_store.save_reminder(reminder, true)?;
        assert!(saved);
        Ok(())
    }
}
