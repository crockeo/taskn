mod commands;
mod opt;
mod taskwarrior;

use std::io;

use opt::{Command, Opt};

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    match opt.command {
        Command::Edit => commands::edit::edit_notes(opt),
        Command::Remind => {
            commands::remind::set_reminders::<commands::remind::MacReminder, _>(opt.args)
        }
    }
}
