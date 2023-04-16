use std::mem::swap;

use crate::{
    game::{
        level::ClearedLines,
        state::{Field, GameState, Square, FIELD_HEIGHT, FIELD_WIDTH},
        tetromino::Tetromino,
    },
    input::Input,
};

use super::finished::Finished;

pub enum Collision {
    OutOfBounds,
    WithBlock,
}

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
            let cleared_lines = self.clear_lines();
            self.state.level.up(&cleared_lines);

            swap(&mut self.state.current, &mut self.state.next);
            self.state.next = Tetromino::next();
        }

        self.state.preview = self.determine_preview();

        None
    }

    fn determine_preview(&self) -> Option<Tetromino> {
        let mut preview = self.state.current.clone();

        preview.move_down();

        if self.check_collision(&preview).is_some() {
            return None;
        }

        while self.check_collision(&preview).is_none() {
            preview.move_down();
        }

        preview.move_up();

        Some(preview)
    }

    fn is_finished(&self) -> bool {
        self.check_collision(&self.state.current).is_some()
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
            Input::Right => self.move_right(),
            Input::Left => self.move_left(),
            Input::Rotate => self.rotate(),
            _ => (),
        }

        match input {
            Input::Down => self.move_down(),
            Input::Drop => self.drop(),
            _ => false,
        }
    }

    fn advance_game(&mut self, already_solidified: bool) -> bool {
        self.state.ticks += 1;

        match self.state.ticks > self.state.level.required_ticks() {
            true => {
                self.state.ticks = 0;
                already_solidified || self.try_move_down()
            }
            false => already_solidified,
        }
    }

    fn try_move_down(&mut self) -> bool {
        match self.try_solidify() {
            true => true,
            false => {
                self.state.current.move_down();
                false
            }
        }
    }

    fn try_solidify(&mut self) -> bool {
        let mut copy = self.state.current.clone();
        copy.move_down();

        if self.check_collision(&copy).is_some() {
            for elem in self.state.current.offset_blocks().iter() {
                self.state.field[elem.vec.y as usize][elem.vec.x as usize] =
                    Square::Occupied(self.state.current.color);
            }

            return true;
        }

        false
    }

    fn check_collision(&self, tetromino: &Tetromino) -> Option<Collision> {
        if Self::is_out_of_bounds(tetromino) {
            return Some(Collision::OutOfBounds);
        }

        if Self::has_collision_with_block(tetromino, &self.state.field) {
            return Some(Collision::WithBlock);
        }

        None
    }

    fn has_collision_with_block(tetromino: &Tetromino, field: &Field) -> bool {
        tetromino.offset_blocks().iter().any(|block| {
            match &field[block.vec.y as usize][block.vec.x as usize] {
                Square::Empty => false,
                Square::Occupied(_) => true,
            }
        })
    }

    fn is_out_of_bounds(tetromino: &Tetromino) -> bool {
        tetromino.offset_blocks().iter().any(|block| {
            block.vec.x as usize >= FIELD_WIDTH
                || block.vec.x < 0
                || block.vec.y as usize >= FIELD_HEIGHT
                || block.vec.y < 0
        })
    }

    fn clear_lines(&mut self) -> ClearedLines {
        self.state
            .field
            .retain(|line| line.iter().any(|square| square == &Square::Empty));

        let cleared_lines = FIELD_HEIGHT - self.state.field.len();

        while self.state.field.len() < FIELD_HEIGHT {
            self.state.field.push_front([Square::Empty; 10]);
        }

        cleared_lines.into()
    }

    fn rotate(&mut self) {
        let original = self.state.current.clone();
        self.state.current.rotate();

        self.kickback(original);
    }

    fn move_right(&mut self) {
        let original = self.state.current.clone();
        self.state.current.move_right();

        self.kickback(original);
    }

    fn move_left(&mut self) {
        let original = self.state.current.clone();
        self.state.current.move_left();

        self.kickback(original);
    }

    fn move_down(&mut self) -> bool {
        self.try_move_down()
    }

    fn drop(&mut self) -> bool {
        while !self.try_move_down() {}

        true
    }

    fn kickback(&mut self, original: Tetromino) {
        if self.check_collision(&self.state.current).is_some() {
            self.state.current = original;
        }
    }
}
