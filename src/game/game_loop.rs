use std::{
    thread,
    time::{Duration, Instant},
};

use crate::ui::Ui;

use super::logic::{End, Logic, TickResult};

const UPDATES_PER_SECOND: u32 = 60;

pub struct GameLoop {
    tick_duration: Duration,
    previous: Instant,
    current: Instant,
    accumulated: Duration,
    game: Logic,
    ui: Ui,
}

impl GameLoop {
    pub fn new(game: Logic, ui: Ui) -> Self {
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

    #[cfg(not(target_os = "windows"))]
    fn idle(&self, difference: Duration) {
        if difference > Duration::from_micros(500) {
            thread::sleep(difference - Duration::from_micros(100));
        }
    }

    #[cfg(target_os = "windows")]
    fn idle(&self, difference: Duration) {
        // Do nothing because Windows timers are very inaccurate.
        // This may increase CPU load but is much more feasible.
    }
}
