use std::{
    collections::VecDeque,
    hash::{Hash, Hasher},
};

use tui::style::Color;

use super::{
    level::Level,
    modes::{finished::Finished, menu::Menu, running::Running},
    tetromino::Tetromino,
};

#[derive(Debug, Clone, PartialEq)]
pub enum GameMode {
    Menu(Menu),
    Running(Box<Running>),
    Finished(Box<Finished>),
}

impl Hash for GameMode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            GameMode::Menu(_) => u64::MAX.hash(state),
            GameMode::Running(running) => running.state.current.hash(state),
            GameMode::Finished(finished) => finished.state.current.hash(state),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Square {
    Empty,
    Occupied(Color),
}

pub const FIELD_HEIGHT: usize = 20;
pub const FIELD_WIDTH: usize = 10;

pub type Field = VecDeque<[Square; FIELD_WIDTH]>;

#[derive(Debug, Clone, PartialEq)]
pub struct GameState {
    pub level: Level,
    pub current: Tetromino,
    pub next: Tetromino,
    pub preview: Option<Tetromino>,
    pub ticks: u32,
    pub field: Field,
}

impl GameState {
    pub fn new(level: u32) -> Self {
        Self {
            level: Level::new(level),
            current: Tetromino::next(),
            next: Tetromino::next(),
            preview: None,
            ticks: 0,
            field: VecDeque::from(vec![[Square::Empty; 10]; 20]),
        }
    }
}
