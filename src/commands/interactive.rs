use std::io;

use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::widgets::Paragraph;
use tui::Terminal;

use crate::opt::Opt;
use crate::taskwarrior::Task;

pub fn execute(opt: Opt) -> io::Result<()> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut taskwarrior_args = opt.args;
    taskwarrior_args.push("(status:pending or status:waiting)".to_string());
    let tasks = Task::get(taskwarrior_args.iter())?;

    // clear screen
    println!("\0{}[2J", 27 as char);
    terminal.draw(move |f| {
        let mut contents = String::new();
        for task in tasks.into_iter() {
            contents.push_str(format!("{}\n", task.description).as_str());
        }
        let paragraph = Paragraph::new(contents);
        f.render_widget(paragraph, f.size());
    })?;

    Ok(())
}
