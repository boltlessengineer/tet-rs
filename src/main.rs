mod core;
mod tui;

use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, KeyCode};

use crate::core::{Direction, Game};

enum Event<I> {
    Input(I),
    Tick,
    FixTimeout,
}

const KEY_TIMEOUT: Duration = Duration::from_millis(200);
const TICK_TIMEOUT: Duration = Duration::from_millis(1000);
const LOCK_DELAY: Duration = Duration::from_millis(500);

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
                        // TODO: don't harddrop after few frames from softdrop
                        game.player
                            .shift(0, game.player.ghost_y - game.player.y, &game.board);
                        game.lock_player();
                    }
                    KeyCode::Char('y') => {
                        game.swap_hold();
                    }
                    KeyCode::Char('h') => {
                        let did_move = game.player.shift(-1, 0, &game.board);
                        game.check_lock(did_move);
                    }
                    KeyCode::Char('j') => {
                        game.player.shift(0, -1, &game.board);
                    }
                    KeyCode::Char('l') => {
                        let did_move = game.player.shift(1, 0, &game.board);
                        game.check_lock(did_move);
                    }
                    KeyCode::Char('a') => {
                        let did_move = game.player.rotate(Direction::L, &game.board);
                        game.check_lock(did_move);
                    }
                    KeyCode::Char('d') => {
                        let did_move = game.player.rotate(Direction::R, &game.board);
                        game.check_lock(did_move);
                    }
                    _ => {}
                },
                Event::Tick => {
                    game.player.shift(0, -1, &game.board);
                }
                Event::FixTimeout => {
                    if let Some(lt) = game.last_touch {
                        let current_time = Instant::now();
                        let duration_since_touch = current_time.duration_since(lt);
                        if duration_since_touch.as_millis() >= LOCK_DELAY.as_millis() {
                            game.lock_player();
                            game.move_after_touch = 0;
                        }
                    }
                }
            }
            if game.player.y == game.player.ghost_y {
                let lock_tx = tx.clone();
                thread::spawn(move || {
                    thread::sleep(LOCK_DELAY);
                    lock_tx.send(Event::FixTimeout).expect("thread error");
                });
                if game.last_touch.is_none() {
                    game.last_touch = Some(Instant::now());
                }
            }
        }
    }

    ui.exit().expect("Error while exiting program");
}
