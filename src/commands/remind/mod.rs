mod eventkit;

use std::io;

use crate::opt::Opt;
use crate::taskwarrior::Task;
use eventkit::{EventStore, Reminder};

pub fn execute(opt: Opt) -> io::Result<()> {
    let mut taskwarrior_args = opt.args;
    taskwarrior_args.push("+remindme".to_string());
    taskwarrior_args.push("(status:pending or status:waiting)".to_string());
    let tasks = Task::get(taskwarrior_args.into_iter())?;

    let mut event_store = EventStore::new().unwrap();
    for task in tasks.into_iter() {
        let reminder = Reminder::new(
            &mut event_store,
            task.description,
            task.uuid,
            task.wait.map(|pdt| pdt.0),
        );
        event_store.save_reminder(reminder, true).unwrap();
    }

    Ok(())
}
