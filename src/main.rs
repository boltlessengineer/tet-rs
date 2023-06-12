mod core;
mod tui;

use std::{sync::mpsc, thread, time::Duration};

use crossterm::event::{self, KeyCode};

use crate::core::{Direction, Game};

enum Event<I> {
    Input(I),
    Tick,
}

const KEY_TIMEOUT: Duration = Duration::from_millis(200);
const TICK_TIMEOUT: Duration = Duration::from_millis(1000);
// const LOCK_TIMEOUT: Duration = Duration::from_millis(500);

fn main() {
    let mut ui = tui::UI::new().expect("Can't initialize TUI");
    // init tick thread and input thread
    let (tx, rx) = mpsc::channel();
    let control_tx = tx.clone();
    let tick_tx = tx.clone();
    // key input thread
    thread::spawn(move || loop {
        // HACK: why timeout here is DAS_TIMEOUT?
        if event::poll(KEY_TIMEOUT).expect("poll error") {
            if let event::Event::Key(key) = event::read().expect("can't read key events") {
                control_tx
                    .send(Event::Input(key))
                    .expect("can't send key events")
            }
        }
    });

    // tick thread
    thread::spawn(move || loop {
        thread::sleep(TICK_TIMEOUT);
        tick_tx.send(Event::Tick).unwrap();
    });

    let mut game = Game::new();

    loop {
        ui.render(&game).unwrap();
        if let Ok(ev) = rx.recv() {
            match ev {
                Event::Input(key) => match key.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char(' ') => {
                        game.player
                            .shift(0, game.player.ghost_y - game.player.y, &game.board);
                        game.lock_player();
                    }
                    KeyCode::Char('y') => {
                        game.swap_hold();
                    }
                    KeyCode::Char('h') => {
                        game.player.shift(-1, 0, &game.board);
                    }
                    KeyCode::Char('j') => {
                        game.player.shift(0, -1, &game.board);
                    }
                    KeyCode::Char('l') => {
                        game.player.shift(1, 0, &game.board);
                    }
                    KeyCode::Char('a') => {
                        game.player.rotate(Direction::L, &game.board);
                    }
                    KeyCode::Char('d') => {
                        game.player.rotate(Direction::R, &game.board);
                    }
                    _ => {}
                },
                Event::Tick => {
                    game.player.shift(0, -1, &game.board);
                }
            }
        }
    }

    ui.exit().expect("Error while exiting program");
}
