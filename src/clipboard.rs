//! Bounded Windows clipboard text access for the global speak shortcut.
use crate::error::{Result, TinyTtsError};

#[cfg(any(windows, test))]
const MAX_CLIPBOARD_BYTES: usize = 2 * 1_048_576;

/// Decode only copied allocation bytes; never scan a native pointer for a terminator.
#[cfg(any(windows, test))]
fn decode_text(bytes: &[u8]) -> Result<Option<String>> {
    if bytes.is_empty() || bytes.len() > MAX_CLIPBOARD_BYTES || !bytes.len().is_multiple_of(2) {
        return Err(TinyTtsError::ClipboardUnavailable(
            "Invalid clipboard allocation size".into(),
        ));
    }
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    let Some(end) = units.iter().position(|unit| *unit == 0) else {
        return Err(TinyTtsError::ClipboardUnavailable(
            "Clipboard text is not terminated".into(),
        ));
    };
    let text = String::from_utf16_lossy(&units[..end]).trim().to_string();
    Ok((!text.is_empty()).then_some(text))
}

#[cfg(all(windows, feature = "native"))]
pub fn read_text() -> Result<Option<String>> {
    use clipboard_win::{formats, raw, Clipboard};
    let error = |e: clipboard_win::ErrorCode| TinyTtsError::ClipboardUnavailable(e.to_string());
    let _clipboard = Clipboard::new().map_err(error)?;
    if !raw::is_format_avail(formats::CF_UNICODETEXT) {
        return Ok(None);
    }
    let size = raw::size(formats::CF_UNICODETEXT)
        .ok_or_else(|| {
            TinyTtsError::ClipboardUnavailable("Cannot determine clipboard allocation".into())
        })?
        .get();
    if size == 0 || size > MAX_CLIPBOARD_BYTES || !size.is_multiple_of(2) {
        return Err(TinyTtsError::ClipboardUnavailable(
            "Invalid clipboard allocation size".into(),
        ));
    }
    let mut bytes = vec![0u8; size];
    // The library copies at most this caller-owned slice length. Keep the clipboard
    // open throughout size/copy so another owner cannot replace its allocation.
    let copied = raw::get(formats::CF_UNICODETEXT, &mut bytes).map_err(error)?;
    bytes.truncate(copied);
    decode_text(&bytes)
}

#[cfg(not(all(windows, feature = "native")))]
pub fn read_text() -> Result<Option<String>> {
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_utf16_and_stops_at_the_bounded_terminator() -> Result<()> {
        assert_eq!(decode_text(&[65, 0, 0, 0, 66, 0])?, Some("A".into()));
        assert_eq!(decode_text(&[0, 0])?, None);
        assert_eq!(
            decode_text(&[0x60, 0x4f, 0x7d, 0x59, 0, 0])?,
            Some("你好".into())
        );
        Ok(())
    }

    #[test]
    fn rejects_odd_unterminated_and_oversized_allocations() {
        for bytes in [&[][..], &[65][..], &[65, 0][..]] {
            assert!(decode_text(bytes).is_err());
        }
        assert!(decode_text(&vec![0; MAX_CLIPBOARD_BYTES + 2]).is_err());
    }
}
