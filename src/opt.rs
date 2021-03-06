use std::env;
use std::str::FromStr;

use structopt::StructOpt;

use crate::commands::Command;

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
    command: String,

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

        let command;
        let args;
        match Command::from_str(&proto_opt.command) {
            Ok(cmd) => {
                command = cmd;
                args = proto_opt.args;
            }
            Err(_) => {
                command = Command::Edit;
                args = [&[proto_opt.command], &proto_opt.args[..]].concat();
            }
        }

        Opt {
            editor,
            file_format: proto_opt.file_format,
            root_dir,
            command,
            args,
        }
    }

    pub fn from_args() -> Self {
        Self::from_proto_opt(ProtoOpt::from_args())
    }
}
