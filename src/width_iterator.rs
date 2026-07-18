use regex::Regex;

use crate::UnicodeWidth;
use std::{borrow::Cow, str::CharIndices, sync::LazyLock};

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
pub(crate) struct WidthIterator<'a, 'b> {
    uw: &'a UnicodeWidth,
    input_str: &'b str,
    input_chars: CharIndices<'b>,
    width: usize,
    max_width: usize,
    pub(crate) input_end_index: Option<usize>,
    output: Option<String>,
    include_at_least_one: bool,
    last_copied_index: usize,
}

impl<'a, 'b> From<WidthIterator<'a, 'b>> for Cow<'b, str> {
    fn from(value: WidthIterator<'a, 'b>) -> Self {
        assert!(value.input_end_index.is_some());
        match value.output {
            None => Cow::Borrowed(&value.input_str[..value.input_end_index.unwrap()]),
            Some(output) => Cow::Owned(output),
        }
    }
}

impl<'a, 'b> WidthIterator<'a, 'b> {
    pub(crate) fn new(uw: &'a UnicodeWidth, input: &'b str) -> Self {
        Self {
            uw,
            input_str: input,
            input_chars: input.char_indices(),
            width: 0,
            max_width: usize::MAX,
            input_end_index: None,
            output: None,
            include_at_least_one: false,
            last_copied_index: 0,
        }
    }

    #[inline]
    pub(crate) fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub(crate) fn set_max_width(&mut self, max_width: usize) {
        self.max_width = max_width;
    }

    #[inline]
    pub(crate) fn set_include_at_least_one(&mut self, include: bool) {
        self.include_at_least_one = include;
    }

    pub(crate) fn consume_all(&mut self) {
        for _ in self.by_ref() {}
        assert!(self.input_end_index.is_some());
    }

    fn next_char(&mut self) -> Option<(usize, char)> {
        let (mut index, mut ch) = self.input_chars.next()?;
        while ch == 0x1B as char
            && self.uw.is_ansi
            && let Some(m) = RE_ANSI.find(&self.input_str[index + 1..])
        {
            for _ in 0..m.len() {
                let _ = self.input_chars.next();
            }
            (index, ch) = self.input_chars.next()?;
        }
        Some((index, ch))
    }

    fn set_input_end_index(&mut self, index: usize) {
        self.input_end_index = Some(index);
        assert!(self.last_copied_index <= index);
        if let Some(ref mut output) = self.output
            && self.last_copied_index < index
        {
            output.push_str(&self.input_str[self.last_copied_index..index]);
            self.last_copied_index = index;
        }
    }
}

impl<'a, 'b> Iterator for WidthIterator<'a, 'b> {
    type Item = (char, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let Some((index, ch)) = self.next_char() else {
            self.set_input_end_index(self.input_str.len());
            return None;
        };
        let ch_width = if let Some(ch_width) = self.uw.char_opt(ch) {
            ch_width
        } else if ch == '\t' && self.uw.tab_size > 0 {
            let tab_size = self.uw.tab_size as usize;
            if self.output.is_none() && self.uw.should_expand_tab {
                self.output = Some(String::with_capacity(self.input_str.len() + tab_size * 4));
                assert_eq!(self.last_copied_index, 0);
            }
            tab_size - (self.width % tab_size)
        } else {
            self.uw.control_size as usize
        };
        let new_width = self.width + ch_width;
        if new_width > self.max_width {
            if index == 0 && self.include_at_least_one {
                // Bypass maximum width check for the first character.
            } else {
                self.set_input_end_index(index);
                return None;
            }
        }
        self.width = new_width;
        if let Some(ref mut output) = self.output
            && ch == '\t'
        {
            if self.last_copied_index < index {
                output.push_str(&self.input_str[self.last_copied_index..index]);
            }
            for _ in 0..ch_width {
                output.push(' ');
            }
            self.last_copied_index = index + 1;
        }
        Some((ch, ch_width))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab() {
        let mut uw = UnicodeWidth::new();
        uw.set_tab_size(4);
        let input = "A\tB";
        let mut iter = WidthIterator::new(&uw, input);
        assert_eq!(iter.next(), Some(('A', 1)));
        assert_eq!(iter.width(), 1);
        assert_eq!(iter.next(), Some(('\t', 3)));
        assert_eq!(iter.width(), 4);
        assert_eq!(iter.next(), Some(('B', 1)));
        assert_eq!(iter.width(), 5);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn ansi_next() {
        let mut uw = UnicodeWidth::new();
        uw.set_ansi(true);
        let mut iter = WidthIterator::new(&uw, "A\x1B[31mZ");
        assert_eq!(iter.next_char(), Some((0, 'A')));
        assert_eq!(iter.next_char(), Some((6, 'Z')));
        assert_eq!(iter.next_char(), None);

        let mut iter = WidthIterator::new(&uw, "A\x1BDZ");
        assert_eq!(iter.next_char(), Some((0, 'A')));
        assert_eq!(iter.next_char(), Some((3, 'Z')));
        assert_eq!(iter.next_char(), None);
    }

    #[test]
    fn ansi_next_at_start_end() {
        let mut uw = UnicodeWidth::new();
        uw.set_ansi(true);
        let mut iter = WidthIterator::new(&uw, "\x1B[31mZ");
        assert_eq!(iter.next_char(), Some((5, 'Z')));
        assert_eq!(iter.next_char(), None);

        uw.set_tab_size(4);
        uw.set_expand_tab(true);
        let mut iter = WidthIterator::new(&uw, "\t\x1B[31m");
        assert_eq!(iter.next_char(), Some((0, '\t')));
        assert_eq!(iter.next_char(), None);
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
        let mut uw = UnicodeWidth::new();
        uw.set_ansi(true);
        for (input, expected) in tests {
            let mut iter = WidthIterator::new(&uw, input);
            let mut actual = String::new();
            while let Some((_, ch)) = iter.next_char() {
                actual.push(ch);
            }
            assert_eq!(actual, expected);
        }
    }
}
