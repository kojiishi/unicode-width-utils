use regex::Regex;
use std::{str::CharIndices, sync::LazyLock};

static RE_ANSI: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"^(?:",
        // CSI sequences (e.g., colors [31m, cursor movement [2J).
        r"\[[0-?]*[ -/]*[@-~]",
        // OSC sequences, ending in either a Bell (\x07) or the String
        // Terminator (ESC \).
        r"|\][^\x1B\x07]*(?:\x1B\\|\x07)",
        // DCS, SOS, PM, and APC strings, which are terminated by the String
        // Terminator (ESC \).
        r"|[PX^_][^\x1B]*\x1B\\",
        // General Escape sequences (2-character sequences like ESC c, ESC 7, etc.)
        r"|[ -/]*[0-~]",
        r")"
    ))
    .unwrap()
});

#[derive(Debug)]
pub(crate) struct WidthSource<'a> {
    input_str: &'a str,
    input_chars: CharIndices<'a>,
    is_ansi: bool,
}

impl<'a> WidthSource<'a> {
    pub(crate) fn new(input_str: &'a str, is_ansi: bool) -> Self {
        Self {
            input_str,
            input_chars: input_str.char_indices(),
            is_ansi,
        }
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.input_str.len()
    }

    #[inline]
    pub(crate) fn slice(&self, start: usize, end: usize) -> &'a str {
        &self.input_str[start..end]
    }
}

impl<'a> Iterator for WidthSource<'a> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        let (mut index, mut ch) = self.input_chars.next()?;
        while ch == 0x1B as char
            && self.is_ansi
            && let Some(m) = RE_ANSI.find(&self.input_str[index + 1..])
        {
            for _ in 0..m.len() {
                let _ = self.input_chars.next();
            }
            (index, ch) = self.input_chars.next()?;
        }
        Some((index, ch))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi_next() {
        let mut source = WidthSource::new("A\x1B[31mZ", true);
        assert_eq!(source.next(), Some((0, 'A')));
        assert_eq!(source.next(), Some((6, 'Z')));
        assert_eq!(source.next(), None);

        let mut source = WidthSource::new("A\x1BDZ", true);
        assert_eq!(source.next(), Some((0, 'A')));
        assert_eq!(source.next(), Some((3, 'Z')));
        assert_eq!(source.next(), None);
    }

    #[test]
    fn ansi_next_at_start_end() {
        let mut source = WidthSource::new("\x1B[31mZ", true);
        assert_eq!(source.next(), Some((5, 'Z')));
        assert_eq!(source.next(), None);

        let mut source = WidthSource::new("\t\x1B[31m", true);
        assert_eq!(source.next(), Some((0, '\t')));
        assert_eq!(source.next(), None);
    }

    #[test]
    fn ansi_variations() {
        let tests = vec![
            // CSI: Colors and Cursor.
            ("\x1b[31mRed Text\x1b[0m", "Red Text"),
            ("\x1b[1;1HHome Position", "Home Position"),
            // Fe: Reset and Cursor Save.
            ("\x1bcReset", "Reset"),
            ("\x1b7Saved", "Saved"),
            // OSC: Title and Hyperlinks.
            ("\x1b]0;Title\x07Visible", "Visible"),
            ("\x1b]8;;http://google.com\x1b\\Link\x1b]8;;\x1b\\", "Link"),
            // DCS/APC/PM: Advanced protocols.
            ("\x1BPqSixelData\x1b\\Clean", "Clean"),
            ("\x1B_Graphics\x1b\\Clean", "Clean"),
            ("\x1B^Privacy\x1b\\Clean", "Clean"),
            // Mixed.
            ("\x1b[31m\x1b]0;Title\x07\x1b[2JSuccess", "Success"),
        ];
        for (input, expected) in tests {
            let source = WidthSource::new(input, true);
            let mut actual = String::new();
            for (_, ch) in source {
                actual.push(ch);
            }
            assert_eq!(actual, expected);
        }
    }
}
