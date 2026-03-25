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

// macOS virtual keycodes
const KC_SPACE: i64 = 49;

impl HotkeyListener {
    pub fn start() -> Result<Self, String> {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            use core_graphics::event::*;
            use core_foundation::runloop::*;
            use std::sync::Mutex;

            let recording = Mutex::new(false);

            let mask = CGEventType::KeyDown as u64
                | CGEventType::KeyUp as u64
                | CGEventType::FlagsChanged as u64;

            // CGEventMaskBit is (1 << event_type)
            let mask = (1u64 << CGEventType::KeyDown as u64)
                | (1u64 << CGEventType::KeyUp as u64)
                | (1u64 << CGEventType::FlagsChanged as u64);

            let tap = CGEventTap::new(
                CGEventTapLocation::HID,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::ListenOnly,
                vec![
                    CGEventType::KeyDown,
                    CGEventType::KeyUp,
                    CGEventType::FlagsChanged,
                ],
                move |_proxy, event_type, event| {
                    let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
                    let flags = event.get_flags();
                    let cmd = flags.contains(CGEventFlags::CGEventFlagCommand);
                    let shift = flags.contains(CGEventFlags::CGEventFlagShift);

                    let mut rec = recording.lock().unwrap();

                    match event_type {
                        CGEventType::KeyDown => {
                            if keycode == KC_SPACE && cmd && shift && !*rec {
                                *rec = true;
                                let _ = tx.send(HotkeyEvent::RecordStart);
                            }
                        }
                        CGEventType::KeyUp => {
                            if keycode == KC_SPACE && *rec {
                                *rec = false;
                                let _ = tx.send(HotkeyEvent::RecordStop);
                            }
                        }
                        CGEventType::FlagsChanged => {
                            // If Cmd or Shift released while recording, stop.
                            if *rec && (!cmd || !shift) {
                                *rec = false;
                                let _ = tx.send(HotkeyEvent::RecordStop);
                            }
                        }
                        _ => {}
                    }

                    None // ListenOnly — never modify events
                },
            );

            match tap {
                Ok(tap) => {
                    // SAFETY: tap.mach_port is a valid CFMachPort from CGEventTapCreate
                    unsafe {
                        let source = tap.mach_port
                            .create_runloop_source(0)
                            .expect("failed to create runloop source");
                        let runloop = CFRunLoop::get_current();
                        runloop.add_source(&source, kCFRunLoopCommonModes);
                        tap.enable();
                        CFRunLoop::run_current();
                    }
                }
                Err(()) => {
                    eprintln!("Failed to create CGEventTap — grant Accessibility permission in System Settings > Privacy & Security > Accessibility");
                }
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
