use std::env;
use std::str::FromStr;

use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "taskn", about = "Taskwarrior task annotation helper")]
struct ProtoOpt {
    /// The editor used to open task notes. If unset, taskn will attempt to use $EDITOR. If $EDITOR
    /// is also unset, taskn will use vi.
    #[structopt(long)]
    editor: Option<String>,

    /// The file format used for task notes.
    #[structopt(long, default_value = "md")]
    file_format: String,

    /// The directory in which task notes are placed. If the directory does not already exist,
    /// taskn will create it.
    #[structopt(long, default_value = "~/.taskn")]
    root_dir: String,

    #[structopt(default_value = "edit")]
    command: Command,

    /// Any remaining arguments are passed along to taskwarrior while selecting tasks.
    args: Vec<String>,
}

pub struct Opt {
    pub editor: String,
    pub file_format: String,
    pub root_dir: String,
    pub command: Command,
    pub args: Vec<String>,
}

impl Opt {
    fn from_proto_opt(proto_opt: ProtoOpt) -> Self {
        let editor = if let Some(editor) = proto_opt.editor {
            editor
        } else if let Ok(editor) = env::var("EDITOR") {
            editor
        } else {
            "vi".to_string()
        };

        let root_dir = shellexpand::tilde(&proto_opt.root_dir).to_string();

        Opt {
            editor,
            file_format: proto_opt.file_format,
            root_dir,
            command: proto_opt.command,
            args: proto_opt.args,
        }
    }

    pub fn from_args() -> Self {
        Self::from_proto_opt(ProtoOpt::from_args())
    }
}

pub enum Command {
    Edit,
    Remind,
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Command::*;
        match s {
            "edit" => Ok(Edit),
            "remind" => Ok(Remind),
            _ => Err(format!("failed to parse Command from '{}'", s)),
        }
    }
}
