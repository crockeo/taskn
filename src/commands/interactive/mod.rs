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
