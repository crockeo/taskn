mod events;

use std::fs::File;
use std::io::{self, Read, Stdout};
use std::path::PathBuf;

use termion::event::Key;
use termion::raw::{IntoRawMode, RawTerminal};
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Modifier, Style};
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
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

type Term = Terminal<TermionBackend<RawTerminal<Stdout>>>;

pub fn execute(opt: Opt) -> io::Result<()> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut taskwarrior_args = opt.args.clone();
    taskwarrior_args.push("(status:pending or status:waiting)".to_string());

    // clear screen
    println!("\0{}[2J", 27 as char);

    let events = Events::new();
    let mut tasks = fetch_tasks(&taskwarrior_args)?;
    let mut mode = Mode::Normal(NormalState::default());
    loop {
        mode.render(&opt, &mut terminal, &tasks)?;
        match events.next()? {
            Event::Key(key) => match key {
                Key::Ctrl('c') => break,
                key => mode = mode.handle_key(key, &tasks)?,
            },
            Event::Resize => continue,
        }
        // TODO: don't just refetch everything when we re-render :)
        tasks = fetch_tasks(&taskwarrior_args)?;
    }

    Ok(())
}

fn fetch_tasks(args: &[String]) -> io::Result<Vec<Task>> {
    // TODO: right now this function ensures our tasks are ordered by time
    // but i'd really prefer to articulate this logic in a way that doesn't
    // produce such far-flung dependencies.
    let mut tasks = Task::get(args.iter())?;
    tasks.sort_by(|a, b| a.wait.partial_cmp(&b.wait).unwrap());
    Ok(tasks)
}

enum Mode {
    Normal(NormalState),
}

impl Mode {
    fn render(&mut self, opt: &Opt, terminal: &mut Term, tasks: &[Task]) -> io::Result<()> {
        match self {
            Mode::Normal(state) => state.render(opt, terminal, tasks),
        }
    }

    fn handle_key(self, key: Key, tasks: &[Task]) -> io::Result<Mode> {
        Ok(match self {
            Mode::Normal(mut state) => {
                state.handle_key(key, tasks)?;
                Mode::Normal(state)
            }
        })
    }
}

struct NormalState {
    list_state: ListState,
}

impl NormalState {
    fn render(&mut self, opt: &Opt, terminal: &mut Term, tasks: &[Task]) -> io::Result<()> {
        let selected = self.selected();
        let contents = {
            let path = PathBuf::new()
                .join(&opt.root_dir)
                .join(&tasks[selected].uuid)
                .with_extension(&opt.file_format);

            match File::open(path) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => "".to_string(),
                Err(e) => return Err(e),
                Ok(mut file) => {
                    let mut buffer = String::new();
                    file.read_to_string(&mut buffer)?;
                    buffer
                }
            }
        };

        terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(frame.size());

            let items: Vec<ListItem> = tasks
                .iter()
                .map(|task| ListItem::new(task.description.as_str()))
                .collect();

            // show all of the tasks
            let list = List::new(items)
                .block(Block::default().title("Tasks").borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::UNDERLINED));

            frame.render_stateful_widget(list, layout[0], &mut self.list_state);

            // preview the current highlighted task's notes
            let paragraph = Paragraph::new(contents)
                .block(Block::default().title("Preview").borders(Borders::ALL));
            frame.render_widget(paragraph, layout[1])
        })
    }

    fn handle_key(&mut self, key: Key, tasks: &[Task]) -> io::Result<()> {
        match key {
            Key::Up => {
                let mut selected = self.selected();
                if selected == 0 {
                    selected = tasks.len();
                }
                self.list_state.select(Some(selected - 1));
            }
            Key::Down => {
                let selected = self.selected();
                self.list_state.select(Some((selected + 1) % tasks.len()));
            }
            _ => {}
        }
        Ok(())
    }

    fn selected(&self) -> usize {
        match self.list_state.selected() {
            None => 0,
            Some(selected) => selected,
        }
    }
}

impl Default for NormalState {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self { list_state }
    }
}
