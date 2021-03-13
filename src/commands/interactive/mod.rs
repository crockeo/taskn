mod events;

use std::io;

use termion::event::Key;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::widgets::Paragraph;
use tui::Terminal;

use crate::opt::Opt;
use crate::taskwarrior::Task;
use events::{Event, Events};

// !!!feature brainstorm!!!
//
// - modal UI; fewer keystrokes = faster interaction.
//   after using a UI for a while, you can learn how to interact
//   so you no longer need to type out full commands
//   - normal mode
//     - up + down to nagivate between notes
//     - enter to open up $EDITOR on the taskn note
//     - "m" or "e" to enter modify/edit mode (which one?)
//     - "a" to enter add mode
//   - edit mode
//     - ESC / Ctrl-F to exit edit more
//     - r to toggle +remindme tag
//     - e change the estimate
//     - u to change urgency
//     - p to change project
//     - t to add/remove tag (normal +/- taskwarrior syntax)
//
// - try to build out a joint estimate + urgency ordering system
//   so that tasks have a consistent order and i can capture
//   top-to-bottom
//
// - preview taskn notes when you select a task

pub fn execute(opt: Opt) -> io::Result<()> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut taskwarrior_args = opt.args;
    taskwarrior_args.push("(status:pending or status:waiting)".to_string());
    let tasks = Task::get(taskwarrior_args.iter())?;

    // clear screen
    println!("\0{}[2J", 27 as char);

    let events = Events::new();
    loop {
        terminal.draw(|f| {
            let mut contents = String::new();
            for task in tasks.iter() {
                contents.push_str(format!("{}\n", task.description).as_str());
            }
            let paragraph = Paragraph::new(contents);
            f.render_widget(paragraph, f.size());
        })?;

        match events.next()? {
            Event::Key(key) => match key {
                Key::Ctrl('c') => break,
                _ => continue,
            },
            Event::Resize => continue,
        }
    }

    Ok(())
}
