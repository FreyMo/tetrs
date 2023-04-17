use std::time::{Duration, Instant};

use crate::ui::Ui;

use super::logic::{End, Logic, TickResult};

const TICKS_PER_SECOND: f64 = 60f64;

pub struct GameLoop {
    tick_duration: Duration,
    previous: Instant,
    current: Instant,
    accumulated: Duration,
    logic: Logic,
    ui: Ui,
}

impl GameLoop {
    pub fn new(logic: Logic, ui: Ui) -> Self {
        let instant = Instant::now();

        GameLoop {
            tick_duration: Duration::from_secs_f64(1.0 / TICKS_PER_SECOND),
            previous: instant,
            current: instant,
            accumulated: Duration::default(),
            logic,
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

            match self.logic.update() {
                TickResult::End(end) => {
                    return Some(end);
                }
                TickResult::Phase(phase) => self.ui.draw(&phase),
            };
        } else {
            self.idle(self.tick_duration - self.accumulated);
        }

        None
    }

    #[cfg(not(target_os = "windows"))]
    fn idle(&self, difference: Duration) {
        if difference > Duration::from_micros(500) {
            std::thread::sleep(difference - Duration::from_micros(100));
        }
    }

    #[cfg(target_os = "windows")]
    fn idle(&self, _: Duration) {
        // Do nothing because Windows timers are very inaccurate.
        // This may increase CPU load but does stabilize frame times.
    }
}
