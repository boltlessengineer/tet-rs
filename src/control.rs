use std::{
    sync::{mpsc::Sender, Arc, Mutex},
    thread,
};

use crossterm::event::{self, KeyCode, KeyEventKind};

use crate::{ControlKind, Event, DAS_TIMEOUT, KEY_TIMEOUT};

pub fn handle_controls(tx: Sender<Event>) {
    let das_scan_left = Arc::new(Mutex::new(false));
    let das_scan_right = Arc::new(Mutex::new(false));
    loop {
        if event::poll(KEY_TIMEOUT).expect("poll error") {
            if let event::Event::Key(key) = event::read().unwrap() {
                use KeyCode::*;
                use KeyEventKind::*;
                let control = match (key.kind, key.code) {
                    (Press, Char('q')) => Some(ControlKind::Quit),
                    (Press, Char('j')) => Some(ControlKind::SoftDrop),
                    (Press, Char(' ')) => Some(ControlKind::HardDrop),
                    (Press, Char('y')) => Some(ControlKind::Hold),
                    (Press, Char('a')) => Some(ControlKind::RotateCC),
                    (Press, Char('s')) => Some(ControlKind::Rotate180),
                    (Press, Char('d')) => Some(ControlKind::Rotate),
                    (Press, Char('h')) => {
                        das_timeout(&tx, &das_scan_left, ControlKind::LeftDasStart);
                        Some(ControlKind::Left)
                    }
                    (Press, Char('l')) => {
                        das_timeout(&tx, &das_scan_right, ControlKind::RightDasStart);
                        Some(ControlKind::Right)
                    }
                    (Release, Char('h')) => {
                        stop_das(&das_scan_left);
                        Some(ControlKind::LeftDasEnd)
                    }
                    (Release, Char('l')) => {
                        stop_das(&das_scan_right);
                        Some(ControlKind::RightDasEnd)
                    }
                    _ => None,
                };
                if let Some(control) = control {
                    tx.send(Event::Control(control))
                        .expect("can't send key events")
                }
            }
        }
    }
}

fn das_timeout(tx: &Sender<Event>, is_scanning: &Arc<Mutex<bool>>, control: ControlKind) {
    let is_scanning = Arc::clone(&is_scanning);
    if !*is_scanning.lock().unwrap() {
        *is_scanning.lock().unwrap() = true;
        let tx_clone = tx.clone();
        thread::spawn(move || {
            thread::sleep(DAS_TIMEOUT);
            if *is_scanning.lock().unwrap() {
                tx_clone.send(Event::Control(control)).unwrap();
            }
        });
    }
}

fn stop_das(is_scanning: &Arc<Mutex<bool>>) {
    let is_scanning = Arc::clone(is_scanning);
    *is_scanning.lock().unwrap() = false;
}
