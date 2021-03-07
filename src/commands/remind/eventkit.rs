//! Provides a high-level wrapper around the eventkit-sys crate defined in this repository.
//! This module allows one to read, create, and update reminders on macOS.
//! Because this module depends on eventkit-sys, do not expect it to compile on non-macOS systems.
use std::ffi::{c_void, CString};
use std::os::raw::c_char;
use std::ptr::null_mut;
use std::slice;
use std::str;
use std::sync::{Condvar, Mutex};

use block::ConcreteBlock;
use chrono::{DateTime, Datelike, TimeZone, Timelike};
use objc::runtime::Object;

// NOTE:
//   - calendarItemWithIdentifier to get a reminder
//   - requires a UUID from the reminder
//   - do we have the UUID after we save a reminder?

#[link(name = "EventKit", kind = "framework")]
extern "C" {}

#[derive(Debug)]
pub enum EKError {
    /// Used when an operation requires some kind of permissions that the user has not provided.
    NoAccess,

    /// General case whenever an NSError is encountered. The String is populated by the NSError's
    /// localizedDescription.
    NSError(String),

    /// Used when an operation attempts to retrieve a value that may not be present.
    NotFound,
}

impl EKError {
    /// The caller of this function must ensure that the *mut Object provided is, in fact, an
    /// NSError nad not some other kind of Object.
    unsafe fn from_ns_error(ns_error: *mut Object) -> EKError {
        let ns_desc = msg_send![ns_error, localizedDescription];
        let desc = from_ns_string(ns_desc);
        EKError::NSError(desc)
    }
}

pub type EKResult<T> = Result<T, EKError>;

pub struct EventStore {
    ek_event_store: *mut Object,
}

impl EventStore {
    pub fn new() -> EKResult<Self> {
        let cls = class!(EKEventStore);
        let mut ek_event_store: *mut Object;
        unsafe {
            ek_event_store = msg_send![cls, alloc];
            ek_event_store = msg_send![ek_event_store, init];
        }

        Ok(Self { ek_event_store })
    }

    pub fn new_with_permission() -> EKResult<Self> {
        let mut event_store = Self::new()?;
        event_store.request_permission()?;
        Ok(event_store)
    }

    pub fn request_permission(&mut self) -> EKResult<()> {
        let has_permission = Mutex::new(false);
        let has_permission_cond = Condvar::new();
        let completion_block = ConcreteBlock::new(|granted: bool, _ns_error: *mut Object| {
            // TODO: handle the ns_error
            let mut lock = has_permission.lock().unwrap();
            *lock = granted;
            has_permission_cond.notify_one();
        });

        let lock = has_permission.lock().unwrap();
        unsafe {
            let _: c_void = msg_send![
                self.ek_event_store,
                requestAccessToEntityType:EKEntityType::Reminder
                completion:completion_block
            ];
        }
        let lock = has_permission_cond.wait(lock).unwrap();

        if !*lock {
            Err(EKError::NoAccess)
        } else {
            Ok(())
        }
    }

    pub fn save_reminder(&mut self, reminder: &Reminder, commit: bool) -> EKResult<bool> {
        let mut ns_error: *mut Object = null_mut();
        let saved: bool;
        unsafe {
            saved = msg_send![
                self.ek_event_store,
                saveReminder:reminder.ek_reminder
                commit:commit
                error:&mut (ns_error) as *mut *mut Object
            ];
        }

        if ns_error != null_mut() {
            unsafe { return Err(EKError::from_ns_error(ns_error)) }
        }

        Ok(saved)
    }

    pub fn get_reminder<S: AsRef<str>>(&mut self, uuid: S) -> EKResult<Reminder> {
        let ns_string = to_ns_string(uuid.as_ref().to_string());
        let ek_reminder: *mut Object;
        unsafe {
            ek_reminder = msg_send![self.ek_event_store, calendarItemWithIdentifier: ns_string];
            let _: *mut Object = msg_send![ns_string, release];
        }

        if ek_reminder == null_mut() {
            Err(EKError::NotFound)
        } else {
            Ok(Reminder { ek_reminder })
        }
    }
}

impl Drop for EventStore {
    fn drop(&mut self) {
        unsafe {
            let _: c_void = msg_send![self.ek_event_store, release];
        }
    }
}

pub struct Reminder {
    ek_reminder: *mut Object,
}

impl Reminder {
    pub fn new<S: AsRef<str>, Tz: TimeZone>(
        event_store: &mut EventStore,
        title: S,
        notes: S,
        date_time: Option<DateTime<Tz>>,
    ) -> Self {
        let cls = class!(EKReminder);
        let ek_reminder: *mut Object;
        unsafe {
            ek_reminder = msg_send![cls, reminderWithEventStore:event_store.ek_event_store];
        }

        let ns_title = to_ns_string(title);
        let ns_notes = to_ns_string(notes);
        unsafe {
            let _: c_void = msg_send![ek_reminder, setTitle: ns_title];
            let _: c_void = msg_send![ek_reminder, setNotes: ns_notes];
        }

        if let Some(date_time) = date_time {
            let ns_date_components = to_ns_date_components(date_time);
            unsafe {
                let _: c_void = msg_send![ek_reminder, setDueDateComponents: ns_date_components];
                let _: c_void = msg_send![ns_date_components, release];
            }
        }

        unsafe {
            let cal: *mut Object =
                msg_send![event_store.ek_event_store, defaultCalendarForNewReminders];
            let _: c_void = msg_send![ek_reminder, setCalendar: cal];
        }
        Self { ek_reminder }
    }

    pub fn uuid(&self) -> String {
        let ns_string: *mut Object;
        unsafe {
            ns_string = msg_send![self.ek_reminder, calendarItemIdentifier];
            from_ns_string(ns_string)
        }
    }
}

impl Drop for Reminder {
    fn drop(&mut self) {
        unsafe {
            let ns_title: *mut Object = msg_send![self.ek_reminder, title];
            let _: c_void = msg_send![ns_title, release];

            let ns_notes: *mut Object = msg_send![self.ek_reminder, notes];
            let _: c_void = msg_send![ns_notes, release];

            let _: c_void = msg_send![self.ek_reminder, release];
        }
    }
}

/// This is defined in Objective C to be:
///
/// ```
/// enum {
///    EKEntityTypeEvent,
///    EKEntityTypeReminder
/// };
/// typedef NSUInteger EKEntityType;
/// ```
///
/// So we just use a similar enum structure here.
#[repr(u64)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum EKEntityType {
    // we don't actually use the Event type ever
    // Event = 0,
    Reminder = 1,
}

/// Converts a str-like to an
/// [NSString](https://developer.apple.com/documentation/foundation/nsstring?language=objc)
/// returning it as a `*mut Object`. It is the responsibility of the caller to free this string.
///
/// # Arguments
///
/// * `s` - The string we want to convert to an NSString. This can be owned or unowned.
fn to_ns_string<S: AsRef<str>>(s: S) -> *mut Object {
    let c_string = CString::new(s.as_ref()).unwrap().into_raw();

    let cls = class!(NSString);
    let mut ns_string: *mut Object;
    unsafe {
        ns_string = msg_send![cls, alloc];
        ns_string = msg_send![ns_string, initWithUTF8String: c_string];
    }

    unsafe {
        let _ = CString::from_raw(c_string);
    }

    ns_string
}

/// Converts an [NSString](https://developer.apple.com/documentation/foundation/nsstring?language=objc)
/// into a [String].
///
/// The provided NSString MUST be UTF8 encoded. This function copies from the NSString, and does
/// not attempt to release it.
unsafe fn from_ns_string(ns_string: *mut Object) -> String {
    let bytes = {
        let bytes: *const c_char = msg_send![ns_string, UTF8String];
        bytes as *const u8
    };
    let len: usize = msg_send![ns_string, lengthOfBytesUsingEncoding:4]; // 4 = UTF8_ENCODING
    let bytes = slice::from_raw_parts(bytes, len);
    str::from_utf8(bytes).unwrap().to_string()
}

/// Converts a [DateTime] of a particular TZ into its
/// [NSDateComponents](https://developer.apple.com/documentation/foundation/nsdatecomponents?language=objc)
/// counterpart.
///
/// # Arguments
///
/// * `date_time` - The datetime we want to convert.
fn to_ns_date_components<Tz: TimeZone>(date_time: DateTime<Tz>) -> *mut Object {
    let mut ns_date_components: *mut Object;
    unsafe {
        ns_date_components = msg_send![class!(NSDateComponents), alloc];
        ns_date_components = msg_send![ns_date_components, init];

        let _: c_void = msg_send![ns_date_components, setYear:date_time.year()];
        let _: c_void = msg_send![ns_date_components, setMonth:date_time.month()];
        let _: c_void = msg_send![ns_date_components, setDay:date_time.day()];
        let _: c_void = msg_send![ns_date_components, setHour:date_time.hour()];
        let _: c_void = msg_send![ns_date_components, setMinute:date_time.minute()];
        let _: c_void = msg_send![ns_date_components, setSecond:date_time.second()];
    }
    ns_date_components
}

#[cfg(test)]
mod tests {
    use chrono::{Local, NaiveDate};

    use super::*;

    #[test]
    fn test_event_store_new() {
        let _ = EventStore::new();
    }

    #[test]
    fn test_event_store_new_with_permission() {
        let _ = EventStore::new_with_permission();
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
    fn test_from_ns_string() {
        let s1 = "hello world".to_string();
        let ns_string = to_ns_string(&s1);
        let s2;
        unsafe {
            s2 = from_ns_string(ns_string);
            let _: c_void = msg_send![ns_string, release];
        }
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_reminder_new() -> EKResult<()> {
        let mut event_store = EventStore::new()?;
        let _ = Reminder::new(
            &mut event_store,
            "a title",
            "a notes",
            Some(Local.from_utc_datetime(&NaiveDate::from_ymd(2021, 5, 01).and_hms(12, 0, 0))),
        );
        Ok(())
    }

    #[test]
    fn test_save_reminder() -> EKResult<()> {
        let mut event_store = EventStore::new()?;
        let reminder = Reminder::new(
            &mut event_store,
            "a title",
            "a notes",
            Some(Local.from_utc_datetime(&NaiveDate::from_ymd(2021, 5, 01).and_hms(12, 0, 0))),
        );
        let saved = event_store.save_reminder(reminder, true)?;
        assert!(saved);
        Ok(())
    }
}
