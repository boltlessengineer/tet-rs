mod control;
mod core;
mod tui;

use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use control::handle_controls;

use crate::core::{Direction, Game};

pub enum Event {
    Control(ControlKind),
    Tick,
    ArrTick,
    FixTimeout,
}

pub enum ControlKind {
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
    Rotate180,

    HardDrop,
    Hold,
}

const ARR_TIMEOUT: Duration = Duration::from_millis(5);
const DAS_TIMEOUT: Duration = Duration::from_millis(122);

const FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / FPS);
const GRAVITY_FRAMES: u8 = 60;

fn main() {
    let mut ui = tui::UI::new().expect("Can't initialize TUI");
    let mut game = Game::new();

    // control thread
    let (tx, rx) = mpsc::channel::<ControlKind>();
    thread::spawn(move || handle_controls(tx));

    let game_start_time = Instant::now();
    let mut previous_frame_time = game_start_time;
    let mut lag = Duration::from_secs(0);

    let mut previous_arr_time = game_start_time;
    let mut gravity_frame_count = 0;

    let mut lag_frame_count = 0;

    loop {
        let mut frame_count = 0;
        let current_time = Instant::now();
        let elasped_time = current_time - previous_frame_time;
        previous_frame_time = current_time;
        lag += elasped_time;

        // time-wise logic
        if current_time - previous_arr_time > ARR_TIMEOUT {
            let das_left = game.das_charge_left.unwrap_or(current_time);
            let das_right = game.das_charge_right.unwrap_or(current_time);
            if das_left < das_right {
                game.player.shift(-1, 0, &game.board);
            } else if das_right < das_left {
                game.player.shift(1, 0, &game.board);
            }
            previous_arr_time = current_time;
        }

        // frame-wise logic
        while lag >= FRAME_DURATION {
            frame_count += 1;
            gravity_frame_count += 1;

            // TODO: check if softDrop enabled
            // if enabled change the statement below
            if gravity_frame_count > GRAVITY_FRAMES {
                game.shift(0, -1);
                gravity_frame_count = 0;
            }

            ui.render(&game).unwrap();
            lag -= FRAME_DURATION;
        }

        // event-wise logic
        if let Ok(control) = rx.try_recv() {
            use ControlKind::*;
            match control {
                Quit => {
                    break;
                }
                Left => {
                    game.shift(-1, 0);
                }
                LeftDasStart => {
                    if game.das_charge_left.is_none() {
                        game.das_charge_left = Some(current_time)
                    }
                }
                LeftDasEnd => {
                    game.das_charge_left = None;
                }
                Right => {
                    game.shift(1, 0);
                }
                RightDasStart => {
                    if game.das_charge_right.is_none() {
                        game.das_charge_right = Some(current_time);
                    }
                }
                RightDasEnd => {
                    game.das_charge_right = None;
                }
                SoftDrop => {
                    game.shift(0, -1);
                }
                Rotate => {
                    game.rotate(Direction::R);
                }
                RotateCC => {
                    game.rotate(Direction::L);
                }
                Rotate180 => {
                    game.rotate(Direction::D);
                }
                HardDrop => {
                    game.lock_player();
                }
                Hold => {
                    game.swap_hold();
                }
            }
        }

        if frame_count > 1 {
            lag_frame_count += frame_count - 1;
        }
    }

    ui.exit().expect("Error while exiting program");

    println!(
        "{} frames lagged for {:?}",
        lag_frame_count,
        game_start_time.elapsed()
    );
}
