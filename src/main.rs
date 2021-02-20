mod commands;
mod opt;
mod taskwarrior;

use std::io;
use std::process::exit;

use opt::Opt;

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    if opt.command == "reminder" {
        commands::remind::set_reminders::<commands::remind::MacReminder, _>(opt.args)?;
        return Ok(());
    } else if opt.command == "edit" {
        commands::edit::edit_notes(opt)?;
    } else {
        eprintln!("Unrecognized command '{}'", opt.command);
        exit(1);
    }

    Ok(())
}
