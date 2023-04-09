use std::sync::mpsc::Receiver;

use crate::input::Input;

use super::{modes::menu::Menu, state::GameMode};

#[derive(PartialEq)]
pub enum End {
    Quit,
    Restart,
}

pub enum TickResult {
    GameMode(GameMode),
    End(End),
}

impl From<Input> for Option<End> {
    fn from(value: Input) -> Self {
        match value {
            Input::Restart => Some(End::Restart),
            Input::Quit => Some(End::Quit),
            _ => None,
        }
    }
}

pub struct Logic {
    inputs: Receiver<Input>,
    mode: GameMode,
}

impl Logic {
    pub const fn new(inputs: Receiver<Input>) -> Self {
        Self {
            inputs,
            mode: GameMode::Menu(Menu {}),
        }
    }

    pub fn update(&mut self) -> TickResult {
        let inputs: Vec<Input> = self.inputs.try_iter().collect();

        if let Some(end) = self.check_for_end(&inputs) {
            return TickResult::End(end);
        }

        match &mut self.mode {
            GameMode::Menu(menu) => {
                if let Some(running) = menu.handle(&inputs) {
                    self.mode = GameMode::Running(running);
                }
            }
            GameMode::Running(running) => {
                if let Some(finished) = running.handle(&inputs) {
                    self.mode = GameMode::Finished(finished);
                }
            }
            GameMode::Finished(finished) => finished.handle(),
        };

        TickResult::GameMode(self.mode.clone())
    }

    fn check_for_end(&self, inputs: &[Input]) -> Option<End> {
        inputs.iter().find_map(|&x| x.into())
    }
}
