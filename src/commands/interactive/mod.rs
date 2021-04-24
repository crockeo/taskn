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

// NOTE: if you're here looking at this code
// and you're thinking to yourself
// "hey this is awful"
// well congratulations, you're in good company.
//
// this is in that early early early stage
// of programming where you're trying to explore
// whatever it is that you want to make
//
// bear with me as it continues to be ugly (for now)

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
//
// let's think about state transitions a little bit more:
//   - some central state concept (CommonState) which things can mutate
//   - a way to build a CommonState from TaskWarrior
//   - and a way to save CommonState to TaskWarrior
//   - sub-state that represents the current mode + additional state associated with that mode
//   - each action produces:
//     - a sub-state (so we can transition)
//     - whether we need to reload entirely
//     - whether we need to flush state

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
    let mut common_state = CommonState::load_from_taskwarrior()?;
    let mut mode: Box<dyn Mode> = Box::new(Normal);
    loop {
        mode.render(&opt, &mut common_state, &mut terminal)?;
        match events.next()? {
            Event::Key(key) => match key {
                Key::Ctrl('c') => break,
                key => {
                    let result = mode.update(&opt, &mut common_state, key)?;
                    if let Some(new_mode) = result.new_mode {
                        mode = new_mode;
                    }
                    if result.should_flush {
                        common_state = common_state.flush_to_taskwarrior()?;
                    } else if result.should_load {
                        common_state = CommonState::load_from_taskwarrior()?;
                    }
                }
            },
            Event::Resize => continue,
        }
    }

    Ok(())
}

struct CommonState {
    list_state: ListState,
    tasks: Vec<Task>,
}

impl CommonState {
    fn load_from_taskwarrior() -> io::Result<Self> {
        let mut tasks = Task::get(["status:pending"].iter())?;
        tasks.sort_by(|a, b| a.estimate.partial_cmp(&b.estimate).unwrap());
        Ok(CommonState {
            list_state: ListState::default(),
            tasks,
        })
    }

    fn flush_to_taskwarrior(self) -> io::Result<Self> {
        for (order, mut task) in self.tasks.into_iter().enumerate() {
            task.estimate = Some(order as i32);
            task.save()?;
        }
        let mut new_self = Self::load_from_taskwarrior()?;
        new_self.list_state.select(self.list_state.selected());
        Ok(new_self)
    }

    fn selected(&self) -> usize {
        match self.list_state.selected() {
            None => 0,
            Some(selected) => selected,
        }
    }
}

struct ActionResult {
    new_mode: Option<Box<dyn Mode>>,
    should_load: bool,
    should_flush: bool,
}

trait Mode {
    fn render(
        &self,
        opt: &Opt,
        common_state: &mut CommonState,
        terminal: &mut Term,
    ) -> io::Result<()>;

    fn update(
        &mut self,
        opt: &Opt,
        common_state: &mut CommonState,
        key: Key,
    ) -> io::Result<ActionResult>;
}

/// The default interactive mode. Does not modify any data. Allows users to look through their
/// tasks alongside their associated taskn notes.
struct Normal;

impl Mode for Normal {
    fn render(
        &self,
        opt: &Opt,
        common_state: &mut CommonState,
        terminal: &mut Term,
    ) -> io::Result<()> {
        common_render(opt, common_state, terminal)
    }

    fn update(
        &mut self,
        opt: &Opt,
        common_state: &mut CommonState,
        key: Key,
    ) -> io::Result<ActionResult> {
        match key {
            Key::Up => {
                let mut selected = common_state.selected();
                if selected == 0 {
                    selected = common_state.tasks.len();
                }
                common_state.list_state.select(Some(selected - 1));
            }
            Key::Down => {
                let selected = common_state.selected();
                common_state
                    .list_state
                    .select(Some((selected + 1) % common_state.tasks.len()));
            }
            // TODO: come up with a keybind that doesn't mean i have to move my right hand off of
            // the arrow keys
            Key::Char('m') => {
                return Ok(ActionResult {
                    new_mode: Some(Box::new(Move)),
                    should_flush: false,
                    should_load: false,
                })
            }
            _ => {}
        }
        Ok(ActionResult {
            new_mode: None,
            should_flush: false,
            should_load: false,
        })
    }
}

/// Allows users to move a selected task (as selected in [Normal] mode) to a different ordering.
/// Used to modifying the order in which tasks appear in the default TaskWarrior report.
struct Move;

impl Mode for Move {
    fn render(
        &self,
        opt: &Opt,
        common_state: &mut CommonState,
        terminal: &mut Term,
    ) -> io::Result<()> {
        // TODO: render this in a way that shows it's different from the normal mode
        common_render(opt, common_state, terminal)
    }

    fn update(
        &mut self,
        opt: &Opt,
        common_state: &mut CommonState,
        key: Key,
    ) -> io::Result<ActionResult> {
        match key {
            // TODO: when these rotate around they have unexpected behavior, special case it to not
            // accidentally rotate the end to the beginning.
            Key::Up => {
                let selected = common_state.selected();
                let next_pos;
                if selected == 0 {
                    next_pos = common_state.tasks.len() - 1;
                } else {
                    next_pos = selected - 1;
                }

                common_state.tasks.swap(selected, next_pos);
                common_state.list_state.select(Some(next_pos));
            }
            Key::Down => {
                let selected = common_state.selected();
                let next_pos = (selected + 1) % common_state.tasks.len();
                common_state.tasks.swap(selected, next_pos);
                common_state.list_state.select(Some(next_pos));
            }
            Key::Char('\n') => {
                return Ok(ActionResult {
                    new_mode: Some(Box::new(Normal)),
                    should_flush: true,
                    should_load: false,
                })
            }
            Key::Esc | Key::Ctrl('f') | Key::Char('q') => {
                // TODO: reset position before popping out of Move
                return Ok(ActionResult {
                    new_mode: Some(Box::new(Normal)),
                    should_flush: false,
                    should_load: false,
                });
            }
            _ => {}
        }

        Ok(ActionResult {
            new_mode: None,
            should_flush: false,
            should_load: false,
        })
    }
}

fn common_render(opt: &Opt, common_state: &mut CommonState, terminal: &mut Term) -> io::Result<()> {
    let selected = common_state.selected();
    let contents = {
        let path = PathBuf::new()
            .join(&opt.root_dir)
            .join(&common_state.tasks[selected].uuid)
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

        let items: Vec<ListItem> = common_state
            .tasks
            .iter()
            .map(|task| ListItem::new(task.description.as_str()))
            .collect();

        // show all of the tasks
        let list = List::new(items)
            .block(Block::default().title("Tasks").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::UNDERLINED));

        frame.render_stateful_widget(list, layout[0], &mut common_state.list_state);

        // preview the current highlighted task's notes
        let paragraph =
            Paragraph::new(contents).block(Block::default().title("Preview").borders(Borders::ALL));
        frame.render_widget(paragraph, layout[1])
    })
}
