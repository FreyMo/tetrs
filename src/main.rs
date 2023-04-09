use game::logic::End;
use tetrs::Tetrs;

mod game;
mod input;
mod tetrs;
mod ui;

fn main() {
    let game = Tetrs::new();
    while game.run() != End::Quit {}
}
