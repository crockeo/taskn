use std::io;
use std::process::Command;

use crate::opt::Opt;
use crate::taskwarrior::Task;

pub fn execute(opt: Opt) -> io::Result<()> {
    let mut taskwarrior_args = opt.args;
    taskwarrior_args.push("+remindme".to_string());
    println!("{:?}", taskwarrior_args);
    let tasks = Task::get(taskwarrior_args.into_iter())?;

    for task in tasks.iter() {
        add_reminder(task)?;
    }

    Ok(())
}

fn add_reminder(task: &Task) -> io::Result<()> {
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

    println!("{}", osascript);

    Command::new("osascript")
        .arg("-e")
        .arg(osascript)
        .output()?;

    Ok(())
}
