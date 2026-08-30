#[cfg(feature = "ansi")]
use regex::Regex;
use std::str::CharIndices;
#[cfg(feature = "ansi")]
use std::sync::LazyLock;

#[cfg(feature = "ansi")]
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
    #[cfg(feature = "ansi")]
    is_ansi: bool,
    #[cfg(feature = "segment")]
    grapheme_iterator: unicode_segmentation::GraphemeIndices<'a>,
    #[cfg(feature = "segment")]
    next_grapheme_boundary: Option<usize>,
}

impl<'a> WidthSource<'a> {
    pub(crate) fn new(input_str: &'a str) -> Self {
        #[cfg(feature = "segment")]
        let (grapheme_iterator, next_grapheme_boundary) = {
            use unicode_segmentation::UnicodeSegmentation;
            let mut iter = input_str.grapheme_indices(true);
            let next_boundary = iter.next().map(|(i, _)| i);
            (iter, next_boundary)
        };

        Self {
            input_str,
            input_chars: input_str.char_indices(),
            #[cfg(feature = "ansi")]
            is_ansi: false,
            #[cfg(feature = "segment")]
            grapheme_iterator,
            #[cfg(feature = "segment")]
            next_grapheme_boundary,
        }
    }

    #[cfg(feature = "ansi")]
    #[inline]
    pub(crate) fn set_ansi(&mut self, is_ansi: bool) {
        self.is_ansi = is_ansi;
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
    type Item = (usize, char, bool);

    fn next(&mut self) -> Option<Self::Item> {
        #[cfg(feature = "ansi")]
        let (mut index, mut ch) = self.input_chars.next()?;
        #[cfg(not(feature = "ansi"))]
        let (index, ch) = self.input_chars.next()?;
        #[cfg(feature = "ansi")]
        while ch == 0x1B as char
            && self.is_ansi
            && let Some(m) = RE_ANSI.find(&self.input_str[index + 1..])
        {
            for _ in 0..m.len() {
                let _ = self.input_chars.next();
            }
            (index, ch) = self.input_chars.next()?;
        }

        #[cfg(feature = "segment")]
        let is_boundary = {
            while let Some(boundary) = self.next_grapheme_boundary
                && boundary < index
            {
                self.next_grapheme_boundary = self.grapheme_iterator.next().map(|(i, _)| i);
            }
            self.next_grapheme_boundary == Some(index)
        };
        #[cfg(not(feature = "segment"))]
        let is_boundary = true;

        Some((index, ch, is_boundary))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next() {
        let mut source = WidthSource::new("AZ");
        assert_eq!(source.next(), Some((0, 'A', true)));
        assert_eq!(source.next(), Some((1, 'Z', true)));
        assert_eq!(source.next(), None);
    }

    #[cfg(feature = "ansi")]
    #[test]
    fn ansi_next() {
        let mut source = WidthSource::new("A\x1B[31mZ");
        source.set_ansi(true);
        assert_eq!(source.next(), Some((0, 'A', true)));
        assert_eq!(source.next(), Some((6, 'Z', true)));
        assert_eq!(source.next(), None);

        let mut source = WidthSource::new("A\x1BDZ");
        source.set_ansi(true);
        assert_eq!(source.next(), Some((0, 'A', true)));
        assert_eq!(source.next(), Some((3, 'Z', true)));
        assert_eq!(source.next(), None);
    }

    #[cfg(feature = "ansi")]
    #[test]
    fn ansi_next_at_start_end() {
        let mut source = WidthSource::new("\x1B[31mZ");
        source.set_ansi(true);
        assert_eq!(source.next(), Some((5, 'Z', true)));
        assert_eq!(source.next(), None);

        let mut source = WidthSource::new("\t\x1B[31m");
        source.set_ansi(true);
        assert_eq!(source.next(), Some((0, '\t', true)));
        assert_eq!(source.next(), None);
    }

    #[cfg(feature = "ansi")]
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
            let mut source = WidthSource::new(input);
            source.set_ansi(true);
            let mut actual = String::new();
            for (_, ch, _) in source {
                actual.push(ch);
            }
            assert_eq!(actual, expected);
        }
    }

    #[cfg(feature = "segment")]
    #[test]
    fn segment_boundaries() {
        // "a\u{301}" is a single grapheme cluster (a with combining acute
        // accent).
        // 'a' starts at index 0 and is a boundary.
        // '\u{301}' starts at index 1 and is NOT a boundary.
        // 'b' starts at index 3 and is a boundary.
        let mut source = WidthSource::new("a\u{301}b");
        assert_eq!(source.next(), Some((0, 'a', true)));
        assert_eq!(source.next(), Some((1, '\u{301}', false)));
        assert_eq!(source.next(), Some((3, 'b', true)));
        assert_eq!(source.next(), None);

        // "\u{1F1FA}\u{1F1F8}" is a single grapheme cluster consisting of
        // U+1F1FA ('\u{1F1FA}') and U+1F1F8 ('\u{1F1F8}').
        // '\u{1F1FA}' starts at index 0 and is a boundary.
        // '\u{1F1F8}' starts at index 4 and is NOT a boundary.
        // 'b' starts at index 8 and is a boundary.
        let mut source = WidthSource::new("\u{1F1FA}\u{1F1F8}b");
        assert_eq!(source.next(), Some((0, '\u{1F1FA}', true)));
        assert_eq!(source.next(), Some((4, '\u{1F1F8}', false)));
        assert_eq!(source.next(), Some((8, 'b', true)));
        assert_eq!(source.next(), None);
    }
}
