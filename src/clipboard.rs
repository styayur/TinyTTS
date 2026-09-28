//! Windows clipboard access (read-only text for the global speak shortcut).

use crate::error::{Result, TinyTtsError};

/// Read the current clipboard text, returning `None` when it is empty or is
/// not text. On non-Windows platforms this is a no-op that returns `None`.
#[cfg(windows)]
pub fn read_text() -> Result<Option<String>> {
    use std::ptr;
    use windows_sys::Win32::System::DataExchange::{
        CloseClipboard, GetClipboardData, OpenClipboard,
    };
    use windows_sys::Win32::System::Memory::{GlobalLock, GlobalUnlock};

    const CF_UNICODETEXT: u32 = 13;

    unsafe {
        if OpenClipboard(ptr::null_mut()) == 0 {
            return Err(TinyTtsError::ClipboardUnavailable(
                "OpenClipboard failed".to_string(),
            ));
        }

        let handle = GetClipboardData(CF_UNICODETEXT);
        if handle.is_null() {
            CloseClipboard();
            return Ok(None);
        }

        let locked = GlobalLock(handle);
        if locked.is_null() {
            CloseClipboard();
            return Err(TinyTtsError::ClipboardUnavailable(
                "GlobalLock failed".to_string(),
            ));
        }

        let wide = locked as *const u16;
        let mut len = 0usize;
        while *wide.add(len) != 0 {
            len += 1;
        }
        let text = if len == 0 {
            String::new()
        } else {
            String::from_utf16_lossy(std::slice::from_raw_parts(wide, len))
        };

        GlobalUnlock(handle);
        CloseClipboard();

        let trimmed = text.trim().to_string();
        Ok((!trimmed.is_empty()).then_some(trimmed))
    }
}

#[cfg(not(windows))]
pub fn read_text() -> Result<Option<String>> {
    Ok(None)
}
