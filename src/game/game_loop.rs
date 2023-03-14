use std::{
    thread,
    time::{Duration, Instant},
};

use crate::ui::Ui;

use super::game::{End, Game, TickResult};

const UPDATES_PER_SECOND: u32 = 60;

pub struct GameLoop {
    tick_duration: Duration,
    previous: Instant,
    current: Instant,
    accumulated: Duration,
    game: Game,
    ui: Ui,
}

impl GameLoop {
    pub fn new(game: Game, ui: Ui) -> Self {
        let instant = Instant::now();

        GameLoop {
            tick_duration: Duration::from_secs_f64(1.0 / UPDATES_PER_SECOND as f64),
            previous: instant,
            current: instant,
            accumulated: Duration::default(),
            game,
            ui,
        }
    }

    pub fn run(&mut self) -> End {
        loop {
            if let Some(end) = self.iterate() {
                return end;
            }
        }
    }

    fn iterate(&mut self) -> Option<End> {
        self.current = Instant::now();

        let mut elapsed = self.current - self.previous;
        self.previous = self.current;

        if elapsed > self.tick_duration {
            elapsed = self.tick_duration;
        }

        self.accumulated += elapsed;

        if self.accumulated >= self.tick_duration {
            self.accumulated -= self.tick_duration;

            match self.game.update() {
                TickResult::End(end) => {
                    return Some(end);
                }
                TickResult::GameMode(mode) => self.ui.draw(&mode),
            };
        } else {
            self.idle(self.tick_duration - self.accumulated);
        }

        None
    }

    fn idle(&self, difference: Duration) {
        // Only for now, should be improved. However, this already reduces CPU load by a lot
        if difference > Duration::from_millis(1) {
            thread::sleep(Duration::from_millis(1));
        }

        // Add conditional sleep for windows
    }
}
