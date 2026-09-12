#![forbid(unsafe_code)]

mod app;
mod game;
mod setup;
mod transport;
mod ui;

fn main() -> () {
    app::run();
}
