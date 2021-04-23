pub mod edit;
pub mod interactive;
pub mod order;
pub mod remind;

use std::io;
use std::str::FromStr;

use crate::opt::Opt;

#[derive(Clone, Copy)]
pub enum Command {
    Edit,
    Interactive,
    Order,
    Remind,
}

impl Command {
    pub fn execute(self, opt: Opt) -> io::Result<()> {
        use Command::*;
        match self {
            Edit => edit::execute(opt),
            Interactive => interactive::execute(opt),
            Order => order::execute(opt),
            Remind => remind::execute(opt),
        }
    }
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Command::*;
        match s {
            "edit" => Ok(Edit),
            "interactive" => Ok(Interactive),
            "order" => Ok(Order),
            "remind" => Ok(Remind),
            _ => Err(format!("failed to parse Command from '{}'", s)),
        }
    }
}
