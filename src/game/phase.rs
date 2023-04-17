use std::hash::{Hash, Hasher};

use self::{finished::Finished, menu::Menu, running::Running};

pub mod finished;
pub mod menu;
pub mod running;

#[derive(Debug, Clone, PartialEq)]
pub enum Phase {
    Menu(Menu),
    Running(Box<Running>),
    Finished(Box<Finished>),
}

impl Hash for Phase {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Phase::Menu(_) => u64::MAX.hash(state),
            Phase::Running(running) => running.state.current.hash(state),
            Phase::Finished(finished) => finished.state.current.hash(state),
        }
    }
}
