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

enum Loop {
    Continue,
    Break,
}

pub struct InputLoop {
    sender: Sender<Input>,
}

impl TryFrom<Event> for Input {
    type Error = ();

    fn try_from(value: Event) -> Result<Self, Self::Error> {
        match value {
            Event::Key(e) => match e.code {
                KeyCode::Up => Ok(Input::Rotate),
                KeyCode::Right => Ok(Input::Right),
                KeyCode::Left => Ok(Input::Left),
                KeyCode::Down => Ok(Input::Down),
                KeyCode::Char('d') => Ok(Input::Drop),
                KeyCode::Char(' ') => Ok(Input::Drop),
                KeyCode::Char('q') => Ok(Input::Quit),
                KeyCode::Char('r') => Ok(Input::Restart),
                KeyCode::Char('c') => match e.modifiers {
                    KeyModifiers::CONTROL => Ok(Input::Quit),
                    _ => Err(()),
                },
                KeyCode::Char(a) => match a {
                    '0'..='9' => Ok(Input::Number(a.to_digit(10).expect("Should not fail"))),
                    _ => Err(()),
                },
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}

impl InputLoop {
    pub fn new(sender: Sender<Input>) -> Self {
        Self { sender }
    }

    pub fn run(&self) {
        loop {
            match self.provide_inputs() {
                Loop::Continue => continue,
                Loop::Break => break,
            };
        }
    }

    fn provide_inputs(&self) -> Loop {
        let input = match crossterm::event::read() {
            Ok(e) => Input::try_from(e).ok(),
            Err(_) => None,
        };

        if let Some(i) = input {
            self.sender.send(i).ok();
            return self.determine_result(&i);
        }

        Loop::Continue
    }

    fn determine_result(&self, input: &Input) -> Loop {
        match input {
            Input::Quit | Input::Restart => Loop::Break,
            _ => Loop::Continue,
        }
    }
}
