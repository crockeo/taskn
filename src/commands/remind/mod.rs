mod eventkit;

use std::io;

use crate::opt::Opt;
use crate::taskwarrior::Task;
use eventkit::{EventStore, Reminder};

pub fn execute(opt: Opt) -> io::Result<()> {
    let mut taskwarrior_args = opt.args;
    taskwarrior_args.push("+remindme".to_string());
    taskwarrior_args.push("(status:pending or status:waiting)".to_string());
    let mut tasks = Task::get(taskwarrior_args.into_iter())?;
    let task_len = tasks.len();

    Task::define_reminder_uda()?;

    let mut event_store = EventStore::new_with_permission().unwrap();
    for (i, task) in tasks.iter_mut().enumerate() {
        let reminder;
        if let Some(taskn_reminder_uuid) = &task.taskn_reminder_uuid {
            reminder = event_store.get_reminder(taskn_reminder_uuid).unwrap();
        } else {
            reminder = Reminder::new(
                &mut event_store,
                &task.description,
                &task.uuid,
                task.wait.clone().map(|pdt| pdt.0),
            );
        }

        event_store
            .save_reminder(&reminder, i == task_len - 1)
            .unwrap();
        task.set_reminder_uuid(reminder.uuid())?;
    }

    Ok(())
}
