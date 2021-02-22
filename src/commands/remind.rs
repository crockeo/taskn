use std::io;
use std::process::Command;
use std::str;

use crate::opt::Opt;
use crate::taskwarrior::Task;

pub fn execute(opt: Opt) -> io::Result<()> {
    let mut taskwarrior_args = opt.args;
    taskwarrior_args.push("+remindme".to_string());
    taskwarrior_args.push("(status:pending or status:waiting)".to_string());
    let tasks = Task::get(taskwarrior_args.into_iter())?;

    for task in tasks.iter() {
        add_reminder(task)?;
    }

    Ok(())
}

fn add_reminder(task: &Task) -> io::Result<()> {
    // if you're reading this, i hate this function as much as you do.
    //
    // i'm looking into alternatives, like using bindgen on Objective-C
    // (see: https://rust-lang.github.io/rust-bindgen/objc.html)
    // to directly interface with the macOS EventKit
    // (see: https://developer.apple.com/documentation/eventkit?language=objc)
    //
    // just bear with me as i get this sorted out.
    let mut osascript = String::new();
    if let Some(wait) = &task.wait {
        let wait = wait.0;
        let date_str = wait.format("%Y-%m-%d");
        let time_str = wait.format("%H:%M");

        osascript.push_str(
            format!(
                "\
set datetime to current date
tell datetime to set {{its year, its month, its day}} to words of \"{date}\"
tell datetime to set {{its hours, its minutes}} to words of \"{time}\"\n",
                date = date_str,
                time = time_str
            )
            .as_str(),
        );
    }

    osascript.push_str(
        "\
tell app \"Reminders\"
    tell list \"Reminders\" of default account
        make new reminder with properties ",
    );
    osascript.push_str(
        format!(
            "{{name:\"{description}\", body:\"{uuid}\"",
            description = task.description,
            uuid = task.uuid,
        )
        .as_str(),
    );

    if task.wait.is_some() {
        osascript.push_str(", remind me date:datetime");
    }
    osascript.push_str(
        "\
}
    end
end",
    );

    // TODO: make this error handling more robust
    let output = Command::new("osascript")
        .arg("-e")
        .arg(&osascript)
        .output()?;
    if !output.status.success() {
        println!("{}", osascript);
        match str::from_utf8(&output.stderr) {
            Ok(err) => print!("Error: {}", err),
            _ => {}
        }
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "failed to excute osascript",
        ));
    }

    Ok(())
}
