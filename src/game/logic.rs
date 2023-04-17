use std::sync::mpsc::Receiver;

use crate::input::Input;

use super::{phase::menu::Menu, phase::Phase};

#[derive(PartialEq)]
pub enum End {
    Quit,
    Restart,
}

pub enum TickResult {
    Phase(Phase),
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
    phase: Phase,
}

impl Logic {
    pub const fn new(inputs: Receiver<Input>) -> Self {
        Self {
            inputs,
            phase: Phase::Menu(Menu {}),
        }
    }

    pub fn update(&mut self) -> TickResult {
        let inputs: Vec<Input> = self.inputs.try_iter().collect();

        if let Some(end) = self.check_for_end(&inputs) {
            return TickResult::End(end);
        }

        match &mut self.phase {
            Phase::Menu(menu) => {
                if let Some(running) = menu.handle(&inputs) {
                    self.phase = Phase::Running(running);
                }
            }
            Phase::Running(running) => {
                if let Some(finished) = running.handle(&inputs) {
                    self.phase = Phase::Finished(finished);
                }
            }
            Phase::Finished(finished) => finished.handle(),
        };

        TickResult::Phase(self.phase.clone())
    }

    fn check_for_end(&self, inputs: &[Input]) -> Option<End> {
        inputs.iter().find_map(|&x| x.into())
    }
}
