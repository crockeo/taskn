mod commands;
mod opt;
mod taskwarrior;

use std::io;

use opt::Opt;

fn main() -> io::Result<()> {
    let opt = Opt::from_args();
    opt.command.execute(opt)
}
