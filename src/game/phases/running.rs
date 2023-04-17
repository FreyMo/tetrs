use std::mem::swap;

use crate::{
    game::{state::GameState, tetromino::Tetromino},
    input::Input,
};

use super::finished::Finished;

#[derive(Debug, Clone, PartialEq)]
pub struct Running {
    pub state: GameState,
}

impl Running {
    pub fn handle(&mut self, inputs: &[Input]) -> Option<Box<Finished>> {
        if self.is_finished() {
            return Some(Box::new(Finished {
                state: self.state.clone(),
            }));
        }

        let solidified = self.handle_inputs(inputs);
        let solidified = self.advance_game(solidified);

        if solidified {
            let cleared_lines = self.state.clear_lines();
            self.state.level.up(&cleared_lines);

            swap(&mut self.state.current, &mut self.state.next);
            self.state.next = Tetromino::next();
        }

        self.state.preview = self.state.determine_preview();

        None
    }

    fn is_finished(&self) -> bool {
        self.state.check_collision(&self.state.current).is_some()
    }

    fn handle_inputs(&mut self, inputs: &[Input]) -> bool {
        let input = inputs.iter().next();

        match input {
            Some(i) => self.handle_input(i),
            None => false,
        }
    }

    fn handle_input(&mut self, input: &Input) -> bool {
        match input {
            Input::Right => self.state.move_right(),
            Input::Left => self.state.move_left(),
            Input::Rotate => self.state.rotate(),
            _ => (),
        }

        match input {
            Input::Down => self.state.move_down(),
            Input::Drop => self.state.drop(),
            _ => false,
        }
    }

    fn advance_game(&mut self, already_solidified: bool) -> bool {
        self.state.ticks += 1;

        match self.state.ticks > self.state.level.required_ticks() {
            true => {
                self.state.ticks = 0;
                already_solidified || self.state.try_move_down()
            }
            false => already_solidified,
        }
    }
}
