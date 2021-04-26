mod events;

use std::io::{self, Stdout};

use termion::event::Key;
use termion::raw::{IntoRawMode, RawTerminal};
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Modifier, Style};
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use tui::Terminal;

use crate::opt::Opt;
use crate::taskwarrior::Task;
use events::{Event, Events};

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
    let mut common_state = CommonState::load_from_taskwarrior(&opt)?;
    let mut mode: Box<dyn Mode> = Box::new(Normal);
    loop {
        mode.render(&mut common_state, &mut terminal)?;
        match events.next()? {
            Event::Key(key) => match key {
                Key::Ctrl('c') => break,
                key => {
                    let result = mode.update(&opt, &mut common_state, key)?;
                    if let Some(new_mode) = result.new_mode {
                        mode = new_mode;
                    }
                    if result.should_flush {
                        common_state = common_state.flush_to_taskwarrior(&opt)?;
                    } else if result.should_load {
                        common_state = CommonState::load_from_taskwarrior(&opt)?;
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
    // TODO: right now we represent the contents of a task on this [CommonState]
    // but it seems like it ought to be on the task instead, since it's specifically
    // that task's contents
    // think about moving this onto the [Task].
    tasks_contents: Vec<(String, String)>,
}

impl CommonState {
    fn load_from_taskwarrior(opt: &Opt) -> io::Result<Self> {
        let mut tasks = Task::get(["status:pending"].iter())?;
        tasks.sort_by(|a, b| a.estimate.partial_cmp(&b.estimate).unwrap());

        let mut list_state = ListState::default();
        if tasks.len() > 0 {
            list_state.select(Some(0));
        }

        let mut tasks_contents = Vec::with_capacity(tasks.len());
        for task in tasks.iter() {
            tasks_contents.push((task.uuid.clone(), task.load_contents(opt)?));
        }

        Ok(CommonState {
            list_state,
            tasks,
            tasks_contents,
        })
    }

    fn flush_to_taskwarrior(self, opt: &Opt) -> io::Result<Self> {
        // need to calculate new_selected before into_iter()
        // because otherwise it would partially move out of self
        // and cause a compiler error
        let mut new_selected = self.selected();
        for (order, mut task) in self.tasks.into_iter().enumerate() {
            task.estimate = Some(order as i32);
            task.save()?;
        }
        let mut new_self = Self::load_from_taskwarrior(opt)?;

        if new_selected >= new_self.tasks.len() {
            new_selected = new_self.tasks.len() - 1;
        }
        new_self.list_state.select(Some(new_selected));
        Ok(new_self)
    }

    fn selected(&self) -> usize {
        match self.list_state.selected() {
            None => 0,
            Some(selected) => selected,
        }
    }

    fn selected_contents(&self) -> &str {
        let selected = self.selected();
        let selected_uuid = &self.tasks[selected].uuid;
        for (uuid, contents) in self.tasks_contents.iter() {
            if selected_uuid == uuid {
                return contents;
            }
        }
        panic!("selected invariant violated");
    }
}

struct ActionResult {
    new_mode: Option<Box<dyn Mode>>,
    should_load: bool,
    should_flush: bool,
}

impl Default for ActionResult {
    fn default() -> Self {
        ActionResult {
            new_mode: None,
            should_load: false,
            should_flush: false,
        }
    }
}

trait Mode {
    fn render(&self, common_state: &mut CommonState, terminal: &mut Term) -> io::Result<()>;

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
    fn render(&self, common_state: &mut CommonState, terminal: &mut Term) -> io::Result<()> {
        terminal.draw(|frame| common_render(frame, common_state, &[Modifier::DIM]))
    }

    fn update(
        &mut self,
        _opt: &Opt,
        common_state: &mut CommonState,
        key: Key,
    ) -> io::Result<ActionResult> {
        let selected = common_state.selected();
        match key {
            Key::Up => {
                if selected > 0 {
                    common_state.list_state.select(Some(selected - 1));
                }
            }
            Key::Down => {
                if selected < common_state.tasks.len() - 1 {
                    common_state.list_state.select(Some(selected + 1));
                }
            }
            Key::Char('d') => {
                return Ok(ActionResult {
                    new_mode: Some(Box::new(Done)),
                    should_flush: false,
                    should_load: false,
                })
            }
            Key::Char('s') => {
                return Ok(ActionResult {
                    new_mode: Some(Box::new(Shift::new(selected))),
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
struct Shift {
    original_pos: usize,
}

impl Shift {
    fn new(current_pos: usize) -> Self {
        Self {
            original_pos: current_pos,
        }
    }
}

impl Mode for Shift {
    fn render(&self, common_state: &mut CommonState, terminal: &mut Term) -> io::Result<()> {
        terminal.draw(|frame| {
            common_render(frame, common_state, &[Modifier::DIM, Modifier::UNDERLINED])
        })
    }

    fn update(
        &mut self,
        _opt: &Opt,
        common_state: &mut CommonState,
        key: Key,
    ) -> io::Result<ActionResult> {
        match key {
            Key::Up => {
                let selected = common_state.selected();
                if selected > 0 {
                    common_state.tasks.swap(selected, selected - 1);
                    common_state.list_state.select(Some(selected - 1));
                }
            }
            Key::Down => {
                let selected = common_state.selected();
                if selected < common_state.tasks.len() - 1 {
                    common_state.tasks.swap(selected, selected + 1);
                    common_state.list_state.select(Some(selected + 1));
                }
            }
            Key::Char('\n') | Key::Char('s') => {
                return Ok(ActionResult {
                    new_mode: Some(Box::new(Normal)),
                    should_flush: true,
                    should_load: false,
                })
            }
            Key::Esc | Key::Ctrl('f') => {
                let selected = common_state.selected();
                let task = common_state.tasks.remove(selected);
                common_state.tasks.insert(self.original_pos, task);
                common_state.list_state.select(Some(self.original_pos));
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

/// Marks a task done as
struct Done;

impl Mode for Done {
    fn render(&self, common_state: &mut CommonState, terminal: &mut Term) -> io::Result<()> {
        terminal.draw(|frame| {
            let layout = default_layout(frame);
            render_tasks(
                frame,
                common_state,
                &[Modifier::DIM, Modifier::CROSSED_OUT],
                layout[0],
            );

            let text = "CONFIRM (ENTER) or CANCEL (ESC)";
            let paragraph = Paragraph::new(text)
                .block(Block::default().title("Mark Done?").borders(Borders::ALL));
            frame.render_widget(paragraph, layout[1]);
        })
    }

    fn update(
        &mut self,
        _opt: &Opt,
        common_state: &mut CommonState,
        key: Key,
    ) -> io::Result<ActionResult> {
        match key {
            Key::Esc | Key::Ctrl('f') => {
                return Ok(ActionResult {
                    new_mode: Some(Box::new(Normal)),
                    should_flush: false,
                    should_load: false,
                })
            }
            Key::Char('\n') => {
                // TODO: mark the current highlighted task as done
                let selected = common_state.selected();
                common_state.tasks[selected].status = "done".to_string();
                return Ok(ActionResult {
                    new_mode: Some(Box::new(Normal)),
                    should_flush: true,
                    should_load: false,
                });
            }
            _ => {}
        }
        Ok(ActionResult::default())
    }
}

type Frame<'a> = tui::Frame<'a, TermionBackend<RawTerminal<Stdout>>>;

fn common_render<'a>(
    frame: &mut Frame<'a>,
    common_state: &mut CommonState,
    selected_modifiers: &[Modifier],
) {
    let layout = default_layout(frame);
    render_tasks(frame, common_state, selected_modifiers, layout[0]);
    render_contents(frame, common_state, layout[1]);
}

fn default_layout<'a>(frame: &mut Frame<'a>) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(frame.size())
}

fn render_tasks<'a>(
    frame: &mut Frame<'a>,
    common_state: &mut CommonState,
    selected_modifiers: &[Modifier],
    area: Rect,
) {
    let items: Vec<ListItem> = common_state
        .tasks
        .iter()
        .map(|task| ListItem::new(task.description.as_str()))
        .collect();

    let mut highlight_style = Style::default();
    for modifier in selected_modifiers.iter() {
        highlight_style = highlight_style.add_modifier(*modifier);
    }
    let list = List::new(items)
        .block(Block::default().title("Tasks").borders(Borders::ALL))
        .highlight_style(highlight_style);

    frame.render_stateful_widget(list, area, &mut common_state.list_state);
}

fn render_contents<'a>(frame: &mut Frame<'a>, common_state: &mut CommonState, area: Rect) {
    // preview the current highlighted task's notes
    let contents = common_state.selected_contents();
    let paragraph =
        Paragraph::new(contents).block(Block::default().title("Preview").borders(Borders::ALL));
    frame.render_widget(paragraph, area)
}
