use std::sync::mpsc::Sender;

use crossterm::event::{Event, KeyCode, KeyModifiers};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Input {
    Left,
    Right,
    Down,
    Rotate,
    Drop,
    Quit,
    Restart,
    Number(u32),
}

pub struct InputLoop {
    sender: Sender<Input>,
    should_stop: bool,
}

impl InputLoop {
    pub fn new(sender: Sender<Input>) -> Self {
        Self {
            sender,
            should_stop: false,
        }
    }

    pub fn run(&mut self) {
        while !self.should_stop {
            self.provide_inputs();
        }
    }

    fn provide_inputs(&mut self) {
        let input = match crossterm::event::read() {
            Ok(e) => self.read_input(&e),
            Err(_) => None,
        };

        if let Some(i) = input {
            self.stop_if_necessary(&i);
            self.sender.send(i).ok();
        }
    }

    fn read_input(&self, event: &Event) -> Option<Input> {
        match event {
            Event::Key(e) => match e.code {
                KeyCode::Up => Some(Input::Rotate),
                KeyCode::Right => Some(Input::Right),
                KeyCode::Left => Some(Input::Left),
                KeyCode::Down => Some(Input::Down),
                KeyCode::Char('d') => Some(Input::Drop),
                KeyCode::Char(' ') => Some(Input::Drop),
                KeyCode::Char('q') => Some(Input::Quit),
                KeyCode::Char('r') => Some(Input::Restart),
                KeyCode::Char('c') => match e.modifiers {
                    KeyModifiers::CONTROL => Some(Input::Quit),
                    _ => None,
                },
                KeyCode::Char(a) => match a {
                    '0'..='9' => Some(Input::Number(a.to_digit(10).expect("Should not fail"))),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    }

    fn stop_if_necessary(&mut self, input: &Input) {
        match input {
            Input::Quit | Input::Restart => {
                self.should_stop = true;
            }
            _ => {}
        };
    }
}
