//! Text segmentation for long-form TTS input.
//!
//! A whole document is never fed to the model at once. This module splits text
//! into chunks small enough for fast first-syllable latency while avoiding the
//! most obvious wrong splits (decimals, abbreviations and initials).

/// Maximum number of Unicode scalar values per chunk (before soft re-splitting).
pub const MAX_CHARS_PER_CHUNK: usize = 200;

/// Common English abbreviations that contain a period but do not end a sentence.
const ABBREVIATIONS: &[&str] = &[
    "mr", "mrs", "ms", "dr", "st", "prof", "sr", "jr", "vs", "etc", "e.g", "i.e", "no", "vol",
    "fig", "cf", "al", "dept", "inc", "ltd", "co", "corp", "approx", "est", "min", "max", "u.s",
    "u.s.a", "u.k", "a.m", "p.m", "www",
];

/// Split `text` into a sequence of chunks.
///
/// Hard breaks are sentence-final punctuation (`。！？；!?;`) and newlines.
/// `.` is treated specially so that `3.14`, `Mr.`, `U.S.A.`, `file.txt` and
/// `example.com` are not treated as sentence boundaries. Chunks that still
/// exceed [`MAX_CHARS_PER_CHUNK`] are re-split on soft breaks (`，,：:、`) and,
/// as a last resort, hard-cut at the limit.
pub fn segment(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut chunks: Vec<String> = Vec::new();
    let mut buf: Vec<char> = Vec::new();

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];

        if is_newline(ch) {
            flush(&mut buf, &mut chunks);
            while i < chars.len() && is_newline(chars[i]) {
                i += 1;
            }
            continue;
        }

        buf.push(ch);

        if is_hard_break(ch, &chars, i) {
            // Group runs like "!!!", "??", "！？" together with the sentence.
            let next_groupable = chars.get(i + 1).copied().is_some_and(is_groupable_punct);
            if !next_groupable {
                flush(&mut buf, &mut chunks);
            }
        } else if buf.len() > MAX_CHARS_PER_CHUNK {
            split_long(&mut buf, &mut chunks);
        }

        i += 1;
    }

    flush(&mut buf, &mut chunks);
    chunks
}

fn is_newline(ch: char) -> bool {
    ch == '\n' || ch == '\r'
}

fn is_hard_break(ch: char, chars: &[char], i: usize) -> bool {
    match ch {
        '。' | '！' | '？' | '；' | '!' | '?' | ';' => true,
        '.' => is_sentence_period(chars, i),
        _ => false,
    }
}

fn is_groupable_punct(ch: char) -> bool {
    matches!(ch, '!' | '?' | '。' | '！' | '？')
}

/// A period only ends a sentence when it is followed by a boundary and is not
/// part of a decimal number, an abbreviation or an initialism.
fn is_sentence_period(chars: &[char], i: usize) -> bool {
    let next = chars.get(i + 1).copied();
    let followed_by_boundary = match next {
        None => true,
        Some(c) => c.is_whitespace() || is_closing_punct(c) || is_cjk(c),
    };
    if !followed_by_boundary {
        return false;
    }

    let prev_is_digit = i > 0 && chars[i - 1].is_ascii_digit();
    let next_is_digit = next.is_some_and(|c| c.is_ascii_digit());
    if prev_is_digit && next_is_digit {
        // "3.14" (decimal numbers, versions, dates).
        return false;
    }

    if is_abbreviation_or_initials(chars, i) {
        return false;
    }

    true
}

fn is_closing_punct(ch: char) -> bool {
    matches!(
        ch,
        ')' | ']' | '}' | '》' | '」' | '』' | '】' | '）' | '"' | '”' | '’' | '\''
    )
}

fn is_cjk(ch: char) -> bool {
    matches!(ch as u32,
        0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF | 0x3040..=0x30FF | 0xAC00..=0xD7AF
    )
}

fn is_abbreviation_or_initials(chars: &[char], i: usize) -> bool {
    // Walk backwards from the period over alphanumerics and dots to recover the
    // current "word" (e.g. "Mr", "U.S.A", "example").
    let mut start = i;
    while start > 0 {
        let c = chars[start - 1];
        if c.is_ascii_alphanumeric() || c == '.' {
            start -= 1;
        } else {
            break;
        }
    }

    let word: String = chars[start..=i]
        .iter()
        .collect::<String>()
        .to_ascii_lowercase();
    let trimmed = word.trim_end_matches('.');
    if ABBREVIATIONS.contains(&trimmed) || ABBREVIATIONS.contains(&word.as_str()) {
        return true;
    }

    // Initialisms such as "U.S.A." or "J.R.R." are single uppercase letters
    // separated by dots.
    let parts: Vec<&str> = word.split('.').filter(|p| !p.is_empty()).collect();
    if parts.len() >= 2
        && parts
            .iter()
            .all(|p| p.len() == 1 && p.chars().next().is_some_and(|c| c.is_ascii_uppercase()))
    {
        return true;
    }

    false
}

fn flush(buf: &mut Vec<char>, chunks: &mut Vec<String>) {
    let s: String = buf.iter().collect();
    let trimmed = s.trim();
    if !trimmed.is_empty() {
        chunks.push(trimmed.to_string());
    }
    buf.clear();
}

fn trim_leading(buf: &mut Vec<char>) {
    let lead = buf.iter().take_while(|c| c.is_whitespace()).count();
    buf.drain(..lead);
}

/// Re-split an over-long buffer. Prefer punctuation, then whitespace, then a
/// hard cut at the character limit.
fn split_long(buf: &mut Vec<char>, chunks: &mut Vec<String>) {
    let punct = last_index_of_any(buf, &['，', ',', '：', ':', '、']);
    let space = last_index_of_any(buf, &[' ']);
    match punct.or(space) {
        Some(idx) => split_at(buf, chunks, idx + 1),
        None => split_at(buf, chunks, MAX_CHARS_PER_CHUNK),
    }
}

fn last_index_of_any(buf: &[char], needle: &[char]) -> Option<usize> {
    (0..buf.len()).rev().find(|&j| needle.contains(&buf[j]))
}

fn split_at(buf: &mut Vec<char>, chunks: &mut Vec<String>, at: usize) {
    let at = at.min(buf.len());
    let left: Vec<char> = buf[..at].to_vec();
    let right: Vec<char> = buf[at..].to_vec();
    let left_s: String = left.iter().collect();
    let trimmed = left_s.trim();
    if !trimmed.is_empty() {
        chunks.push(trimmed.to_string());
    }
    *buf = right;
    trim_leading(buf);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input() {
        assert_eq!(segment(""), Vec::<String>::new());
        assert_eq!(segment("   \n \t "), Vec::<String>::new());
    }

    #[test]
    fn chinese_sentences() {
        let chunks = segment("你好世界。今天天气很好！我们出去玩？");
        assert_eq!(chunks, vec!["你好世界。", "今天天气很好！", "我们出去玩？"]);
    }

    #[test]
    fn english_sentences() {
        let chunks = segment("Hello world. This is a test! Really?");
        assert_eq!(chunks, vec!["Hello world.", "This is a test!", "Really?"]);
    }

    #[test]
    fn mixed_chinese_english() {
        let chunks = segment("你好。Hello world. 再见！");
        assert_eq!(chunks, vec!["你好。", "Hello world.", "再见！"]);
    }

    #[test]
    fn multiple_newlines_collapse() {
        assert_eq!(segment("a\n\n\nb"), vec!["a", "b"]);
    }

    #[test]
    fn long_sentence_respects_limit() {
        let long = "word ".repeat(80); // 400 chars
        let chunks = segment(&long);
        assert!(!chunks.is_empty());
        for c in &chunks {
            assert!(
                c.chars().count() <= MAX_CHARS_PER_CHUNK,
                "chunk too long: {c}"
            );
        }
    }

    #[test]
    fn consecutive_punctuation_stays_grouped() {
        let chunks = segment("Hello!!!How are you??");
        assert_eq!(chunks, vec!["Hello!!!", "How are you??"]);
    }

    #[test]
    fn decimal_is_not_a_break() {
        let chunks = segment("Pi is 3.14 approximately.");
        assert_eq!(chunks, vec!["Pi is 3.14 approximately."]);
    }

    #[test]
    fn abbreviation_is_not_a_break() {
        let chunks = segment("Mr. Smith went home.");
        assert_eq!(chunks, vec!["Mr. Smith went home."]);
    }

    #[test]
    fn initials_are_not_a_break() {
        let chunks = segment("He lives in the U.S.A. today.");
        assert_eq!(chunks, vec!["He lives in the U.S.A. today."]);
    }

    #[test]
    fn filename_and_url_are_not_split() {
        let chunks = segment("Open file.txt and visit example.com for details.");
        assert_eq!(
            chunks,
            vec!["Open file.txt and visit example.com for details."]
        );
    }
}
