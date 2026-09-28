//! Global hotkey (Ctrl+Alt+S) support.
//!
//! The hotkey is registered on a background thread that owns a Windows message
//! loop. Pressing it posts an event through a channel that the UI polls once
//! per frame, so it works even while the main window is minimized.
//!
//! On non-Windows platforms this module provides a no-op stub so the rest of the
//! code can keep the same interface.

use std::time::Duration;

/// Events emitted by the hotkey listener.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    SpeakClipboard,
}

#[cfg(windows)]
mod imp {
    use super::HotkeyEvent;
    use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
    use std::thread;

    use windows_sys::Win32::Foundation::{HWND, WPARAM};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE, WM_HOTKEY,
    };

    const HOTKEY_ID: i32 = 1;
    const MOD_ALT: u32 = 0x0001;
    const MOD_CONTROL: u32 = 0x0002;
    const VK_S: u32 = 0x53;

    pub struct GlobalHotkey {
        rx: Receiver<HotkeyEvent>,
        stop: Sender<()>,
    }

    impl GlobalHotkey {
        pub fn register() -> Option<Self> {
            let (tx, rx) = channel();
            let (stop, stop_rx) = channel();

            let spawned = thread::Builder::new()
                .name("tinytts-hotkey".to_string())
                .spawn(move || unsafe {
                    if RegisterHotKey(0 as HWND, HOTKEY_ID, MOD_CONTROL | MOD_ALT, VK_S) == 0 {
                        return;
                    }

                    let mut msg: MSG = std::mem::zeroed();
                    loop {
                        match stop_rx.try_recv() {
                            Ok(_) | Err(TryRecvError::Disconnected) => break,
                            Err(TryRecvError::Empty) => {}
                        }

                        if PeekMessageW(&mut msg, 0 as HWND, 0, 0, PM_REMOVE) != 0 {
                            if msg.message == WM_HOTKEY && msg.wParam == HOTKEY_ID as WPARAM {
                                let _ = tx.send(HotkeyEvent::SpeakClipboard);
                            }
                            TranslateMessage(&msg);
                            DispatchMessageW(&msg);
                        } else {
                            thread::sleep(Duration::from_millis(10));
                        }
                    }

                    UnregisterHotKey(0 as HWND, HOTKEY_ID);
                })
                .ok()?;

            if spawned.is_finished() {
                // Registration failed and the thread already exited.
                return None;
            }

            Some(Self { rx, stop })
        }

        pub fn try_recv(&self) -> Option<HotkeyEvent> {
            match self.rx.try_recv() {
                Ok(ev) => Some(ev),
                Err(_) => None,
            }
        }
    }

    impl Drop for GlobalHotkey {
        fn drop(&mut self) {
            let _ = self.stop.send(());
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use super::HotkeyEvent;

    pub struct GlobalHotkey;

    impl GlobalHotkey {
        pub fn register() -> Option<Self> {
            None
        }
        pub fn try_recv(&self) -> Option<HotkeyEvent> {
            None
        }
    }
}

pub use imp::GlobalHotkey;
