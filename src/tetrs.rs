use std::{
    io::stdout,
    marker::PhantomData,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crossterm::{
    cursor, execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, EnterAlternateScreen, LeaveAlternateScreen,
    },
};

use crate::{
    game::{
        cycle::GameLoop,
        logic::{End, Logic},
    },
    input::{Input, InputLoop},
    ui::Ui,
};

pub struct Tetrs {
    phantom: PhantomData<()>,
}

impl Tetrs {
    pub fn new() -> Self {
        enable_raw_mode().unwrap();
        execute!(stdout(), EnterAlternateScreen, cursor::Hide).unwrap();

        Self {
            phantom: PhantomData,
        }
    }

    pub fn run(&self) -> End {
        execute!(stdout(), Clear(crossterm::terminal::ClearType::All)).unwrap();

        let (sender, receiver): (Sender<Input>, Receiver<Input>) = mpsc::channel();

        let input_thread = thread::spawn(|| {
            InputLoop::new(sender).run();
        });

        let end = GameLoop::new(Logic::new(receiver), Ui::default()).run();

        input_thread.join().unwrap();

        end
    }
}

impl Drop for Tetrs {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        execute!(stdout(), LeaveAlternateScreen, cursor::Show).unwrap();
    }
}
