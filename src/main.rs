mod core;
mod tui;

use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, KeyCode, KeyEventKind};

use crate::core::{Direction, Game};

enum Event {
    Control(ControlKind),
    Tick,
    ArrTick,
    FixTimeout,
}

enum ControlKind {
    Quit,

    Left,
    LeftDasStart,
    LeftDasEnd,

    Right,
    RightDasStart,
    RightDasEnd,

    SoftDrop,

    Rotate,
    RotateCC,

    HardDrop,
    Hold,
}

const KEY_TIMEOUT: Duration = Duration::from_millis(10000);
const TICK_TIMEOUT: Duration = Duration::from_millis(1000);
const LOCK_DELAY: Duration = Duration::from_millis(500);
const ARR_TIMEOUT: Duration = Duration::from_millis(5);
const DAS_TIMEOUT: Duration = Duration::from_millis(120);

fn main() {
    let mut ui = tui::UI::new().expect("Can't initialize TUI");
    // init tick thread and input thread
    let (tx, rx) = mpsc::channel();
    let control_tx = tx.clone();
    let tick_tx = tx.clone();
    // key input thread
    thread::spawn(move || {
        let das_scan_left = Arc::new(Mutex::new(false));
        let das_scan_right = Arc::new(Mutex::new(false));
        loop {
            if event::poll(KEY_TIMEOUT).expect("poll error") {
                if let event::Event::Key(key) = event::read().expect("can't read key events") {
                    let control = if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') => Some(ControlKind::Quit),
                            KeyCode::Char('j') => Some(ControlKind::SoftDrop),
                            KeyCode::Char(' ') => Some(ControlKind::HardDrop),
                            KeyCode::Char('y') => Some(ControlKind::Hold),
                            KeyCode::Char('a') => Some(ControlKind::RotateCC),
                            KeyCode::Char('d') => Some(ControlKind::Rotate),
                            KeyCode::Char('h') => {
                                let is_scanning = Arc::clone(&das_scan_left);
                                if !*is_scanning.lock().unwrap() {
                                    *is_scanning.lock().unwrap() = true;
                                    let tx_clone = control_tx.clone();
                                    thread::spawn(move || {
                                        thread::sleep(DAS_TIMEOUT);
                                        if *is_scanning.lock().unwrap() {
                                            tx_clone
                                                .send(Event::Control(ControlKind::LeftDasStart))
                                                .unwrap();
                                        }
                                    });
                                }
                                Some(ControlKind::Left)
                            }
                            KeyCode::Char('l') => {
                                let is_scanning = Arc::clone(&das_scan_right);
                                if !*is_scanning.lock().unwrap() {
                                    *is_scanning.lock().unwrap() = true;
                                    let tx_clone = control_tx.clone();
                                    thread::spawn(move || {
                                        thread::sleep(DAS_TIMEOUT);
                                        if *is_scanning.lock().unwrap() {
                                            tx_clone
                                                .send(Event::Control(ControlKind::RightDasStart))
                                                .unwrap();
                                        }
                                    });
                                }
                                Some(ControlKind::Right)
                            }
                            _ => None,
                        }
                    } else if key.kind == KeyEventKind::Release {
                        match key.code {
                            KeyCode::Char('h') => {
                                let is_scanning = Arc::clone(&das_scan_left);
                                *is_scanning.lock().unwrap() = false;
                                Some(ControlKind::LeftDasEnd)
                            }
                            KeyCode::Char('l') => {
                                let is_scanning = Arc::clone(&das_scan_right);
                                *is_scanning.lock().unwrap() = false;
                                Some(ControlKind::RightDasEnd)
                            }
                            _ => None,
                        }
                    } else {
                        None
                    };
                    if let Some(e) = control {
                        control_tx
                            .send(Event::Control(e))
                            .expect("can't send key events")
                    }
                }
            }
        }
    });

    // tick thread
    thread::spawn(move || loop {
        thread::sleep(TICK_TIMEOUT);
        tick_tx.send(Event::Tick).unwrap();
    });

    let mut game = Game::new();

    let left_charged = Arc::new(Mutex::new(false));
    let right_charged = Arc::new(Mutex::new(false));
    let mut das_dir = Direction::Z;
    loop {
        let left_charged = Arc::clone(&left_charged);
        let right_charged = Arc::clone(&right_charged);
        ui.render(&game).unwrap();
        if let Ok(ev) = rx.recv() {
            match ev {
                Event::Control(control) => match control {
                    ControlKind::Quit => {
                        break;
                    }
                    ControlKind::HardDrop => {
                        // TODO: don't harddrop after few frames from softdrop
                        game.player.das_shift(Direction::D, &game.board);
                        game.lock_player();
                    }
                    ControlKind::Hold => {
                        game.swap_hold();
                    }
                    ControlKind::Left => {
                        let did_move = game.player.shift(-1, 0, &game.board);
                        game.check_lock(did_move);
                    }
                    ControlKind::SoftDrop => {
                        game.player.shift(0, -1, &game.board);
                    }
                    ControlKind::Right => {
                        let did_move = game.player.shift(1, 0, &game.board);
                        game.check_lock(did_move);
                    }
                    ControlKind::RotateCC => {
                        let did_move = game.player.rotate(Direction::L, &game.board);
                        game.check_lock(did_move);
                    }
                    ControlKind::Rotate => {
                        let did_move = game.player.rotate(Direction::R, &game.board);
                        game.check_lock(did_move);
                    }
                    ControlKind::LeftDasStart => {
                        if !*left_charged.lock().unwrap() {
                            *left_charged.lock().unwrap() = true;
                            let tx_clone = tx.clone();
                            thread::spawn(move || loop {
                                if !*left_charged.lock().unwrap() {
                                    break;
                                }
                                tx_clone.send(Event::ArrTick).unwrap();
                                thread::sleep(ARR_TIMEOUT);
                            });
                        }
                        das_dir = Direction::L;
                    }
                    ControlKind::RightDasStart => {
                        if !*right_charged.lock().unwrap() {
                            *right_charged.lock().unwrap() = true;
                            let tx_clone = tx.clone();
                            thread::spawn(move || loop {
                                if !*right_charged.lock().unwrap() {
                                    break;
                                }
                                tx_clone.send(Event::ArrTick).unwrap();
                                thread::sleep(ARR_TIMEOUT);
                            });
                        }
                        das_dir = Direction::R;
                    }
                    ControlKind::LeftDasEnd => {
                        *left_charged.lock().unwrap() = false;
                        if *right_charged.lock().unwrap() {
                            // re-send LeftDasStart to restart ARR tick
                            let tx_clone = tx.clone();
                            tx_clone
                                .send(Event::Control(ControlKind::RightDasStart))
                                .unwrap();
                        }
                    }
                    ControlKind::RightDasEnd => {
                        *right_charged.lock().unwrap() = false;
                        if *left_charged.lock().unwrap() {
                            // re-send LeftDasStart to restart ARR tick
                            let tx_clone = tx.clone();
                            tx_clone
                                .send(Event::Control(ControlKind::LeftDasStart))
                                .unwrap();
                        }
                    }
                },
                Event::Tick => {
                    game.player.shift(0, -1, &game.board);
                }
                Event::ArrTick => match das_dir {
                    Direction::L => {
                        game.player.shift(-1, 0, &game.board);
                    }
                    Direction::R => {
                        game.player.shift(1, 0, &game.board);
                    }
                    _ => unreachable!(),
                },
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
