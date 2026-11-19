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
    use windows_sys::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};

    const CF_UNICODETEXT: u32 = 13;

    // SAFETY: clipboard ownership remains open while its non-null HGLOBAL is locked.
    // GlobalSize bounds the aligned UTF-16 allocation; a terminator must occur inside
    // that allocation before text is copied. Every exit after locking unlocks the
    // handle and closes the clipboard; no borrowed clipboard storage escapes.
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

        let units = GlobalSize(handle) / std::mem::size_of::<u16>();
        if units == 0 || units > 1_048_576 {
            GlobalUnlock(handle);
            CloseClipboard();
            return Err(TinyTtsError::ClipboardUnavailable(
                "Clipboard text exceeds bounds".into(),
            ));
        }
        let buffer = std::slice::from_raw_parts(locked as *const u16, units);
        let Some(len) = buffer.iter().position(|value| *value == 0) else {
            GlobalUnlock(handle);
            CloseClipboard();
            return Err(TinyTtsError::ClipboardUnavailable(
                "Clipboard text is not terminated".into(),
            ));
        };
        let wide = locked as *const u16;
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
