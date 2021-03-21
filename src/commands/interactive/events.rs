use std::io;
use std::sync::mpsc;
use std::thread;

use signal_hook::consts::signal::SIGWINCH;
use signal_hook::iterator::Signals;
use termion::event::Key;
use termion::input::TermRead;

pub enum Event {
    Key(Key),
    Resize,
}

pub struct Events {
    rx: mpsc::Receiver<Event>,

    _input_thread: thread::JoinHandle<()>,
    _signal_thread: thread::JoinHandle<()>,
}

impl Events {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            rx,
            _input_thread: make_input_thread(tx.clone()),
            _signal_thread: make_signal_thread(tx.clone()),
        }
    }

    pub fn next(&self) -> io::Result<Event> {
        Ok(self.rx.recv().unwrap())
    }
}

fn make_input_thread(tx: mpsc::Sender<Event>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let stdin = io::stdin();
        for key in stdin.keys() {
            tx.send(Event::Key(key.unwrap())).unwrap();
        }
    })
}

fn make_signal_thread(tx: mpsc::Sender<Event>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut signals = Signals::new(&[SIGWINCH]).unwrap();
        loop {
            for signal in &mut signals {
                if signal == SIGWINCH {
                    tx.send(Event::Resize).unwrap();
                }
            }
        }
    })
}
