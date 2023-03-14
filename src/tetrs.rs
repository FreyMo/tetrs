use std::{
    io::stdout,
    marker::PhantomData,
    sync::{
        atomic::AtomicBool,
        mpsc::{self, Receiver, Sender},
        Arc,
    },
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
        game::{End, Game},
        game_loop::GameLoop,
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

        let should_stop = Arc::new(AtomicBool::new(false));
        let should_stop_game = should_stop.clone();

        let (sender, receiver): (Sender<Input>, Receiver<Input>) = mpsc::channel();

        let input_thread = thread::spawn(move || {
            InputLoop::new(sender, should_stop).run();
        });

        let end = GameLoop::new(Game::new(receiver, should_stop_game), Ui::default()).run();

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
