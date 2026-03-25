use rdev::{listen, Event, EventType, Key};
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone)]
pub enum HotkeyEvent {
    RecordStart,
    RecordStop,
}

pub struct HotkeyListener {
    receiver: mpsc::Receiver<HotkeyEvent>,
}

impl HotkeyListener {
    pub fn start() -> Result<Self, String> {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut cmd_held = false;
            let mut shift_held = false;
            let mut space_held = false;
            let mut recording = false;

            let callback = move |event: Event| {
                match event.event_type {
                    EventType::KeyPress(key) => {
                        match key {
                            Key::MetaLeft | Key::MetaRight => cmd_held = true,
                            Key::ShiftLeft | Key::ShiftRight => shift_held = true,
                            Key::Space => {
                                if cmd_held && shift_held && !space_held {
                                    space_held = true;
                                    if !recording {
                                        recording = true;
                                        let _ = tx.send(HotkeyEvent::RecordStart);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    EventType::KeyRelease(key) => {
                        match key {
                            Key::MetaLeft | Key::MetaRight => {
                                cmd_held = false;
                                if recording {
                                    recording = false;
                                    space_held = false;
                                    let _ = tx.send(HotkeyEvent::RecordStop);
                                }
                            }
                            Key::ShiftLeft | Key::ShiftRight => {
                                shift_held = false;
                                if recording {
                                    recording = false;
                                    space_held = false;
                                    let _ = tx.send(HotkeyEvent::RecordStop);
                                }
                            }
                            Key::Space => {
                                space_held = false;
                                if recording {
                                    recording = false;
                                    let _ = tx.send(HotkeyEvent::RecordStop);
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            };

            if let Err(e) = listen(callback) {
                eprintln!("rdev listen error: {:?}", e);
            }
        });

        Ok(HotkeyListener { receiver: rx })
    }

    pub fn try_recv(&self) -> Option<HotkeyEvent> {
        self.receiver.try_recv().ok()
    }

    pub fn recv(&self) -> Result<HotkeyEvent, mpsc::RecvError> {
        self.receiver.recv()
    }
}
