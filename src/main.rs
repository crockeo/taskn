#[macro_use]
extern crate objc;

mod commands;
mod opt;
mod taskwarrior;

use std::io;

use commands::{edit, remind};
use opt::{Command, Opt};

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    match opt.command {
        Command::Edit => edit::execute(opt),
        Command::Remind => remind::execute(opt),
    }
}
