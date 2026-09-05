use crate::{UnicodeWidth, WidthSource};
use std::borrow::Cow;

#[derive(Debug)]
pub(crate) struct WidthIterator<'a, 'b> {
    uw: &'a UnicodeWidth,
    source: WidthSource<'b>,
    width: usize,
    max_width: usize,
    pub(crate) input_end_index: Option<usize>,
    output: Option<String>,
    include_at_least_one: bool,
    last_copied_index: usize,
    #[cfg(feature = "segment")]
    last_boundary_index: usize,
    #[cfg(feature = "segment")]
    last_boundary_width: usize,
}

impl<'a, 'b> From<WidthIterator<'a, 'b>> for Cow<'b, str> {
    fn from(value: WidthIterator<'a, 'b>) -> Self {
        assert!(value.input_end_index.is_some());
        match value.output {
            None => Cow::Borrowed(value.source.slice(0, value.input_end_index.unwrap())),
            Some(output) => Cow::Owned(output),
        }
    }
}

impl<'a, 'b> WidthIterator<'a, 'b> {
    pub(crate) fn new(uw: &'a UnicodeWidth, input: &'b str) -> Self {
        #[allow(unused_mut)]
        let mut source = WidthSource::new(input);
        #[cfg(feature = "ansi")]
        source.set_ansi(uw.is_ansi);

        Self {
            uw,
            source,
            width: 0,
            max_width: usize::MAX,
            input_end_index: None,
            output: None,
            include_at_least_one: false,
            last_copied_index: 0,
            #[cfg(feature = "segment")]
            last_boundary_index: 0,
            #[cfg(feature = "segment")]
            last_boundary_width: 0,
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

    #[inline]
    fn should_stop_before(&self, new_width: usize, _index: usize) -> bool {
        if new_width <= self.max_width {
            return false;
        }
        if self.include_at_least_one {
            #[cfg(feature = "segment")]
            {
                return self.last_boundary_index != 0;
            }
            #[cfg(not(feature = "segment"))]
            {
                return _index != 0;
            }
        }
        true
    }

    pub(crate) fn consume_all(&mut self) {
        for _ in self.by_ref() {}
        assert!(self.input_end_index.is_some());
    }

    fn set_input_end_index(&mut self, index: usize) {
        self.input_end_index = Some(index);
        assert!(self.last_copied_index <= index);
        if let Some(ref mut output) = self.output
            && self.last_copied_index < index
        {
            output.push_str(self.source.slice(self.last_copied_index, index));
            self.last_copied_index = index;
        }
    }
}

impl<'a, 'b> Iterator for WidthIterator<'a, 'b> {
    type Item = (char, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let Some((index, ch, is_boundary)) = self.source.next() else {
            self.set_input_end_index(self.source.len());
            return None;
        };

        #[cfg(feature = "segment")]
        if is_boundary {
            self.last_boundary_index = index;
            self.last_boundary_width = self.width;
        }
        #[cfg(not(feature = "segment"))]
        let _ = is_boundary;

        let ch_width = if let Some(ch_width) = self.uw.char_opt(ch) {
            ch_width
        } else if ch == '\t' && self.uw.tab_size > 0 {
            let tab_size = self.uw.tab_size as usize;
            if self.output.is_none() && self.uw.should_expand_tab {
                self.output = Some(String::with_capacity(self.source.len() + tab_size * 4));
                assert_eq!(self.last_copied_index, 0);
            }
            tab_size - (self.width % tab_size)
        } else {
            self.uw.control_size as usize
        };
        let new_width = self.width + ch_width;
        if self.should_stop_before(new_width, index) {
            #[cfg(feature = "segment")]
            {
                self.width = self.last_boundary_width;
                self.set_input_end_index(self.last_boundary_index);
                return None;
            }
            #[cfg(not(feature = "segment"))]
            {
                self.set_input_end_index(index);
                return None;
            }
        }
        self.width = new_width;
        if let Some(ref mut output) = self.output
            && ch == '\t'
        {
            if self.last_copied_index < index {
                output.push_str(self.source.slice(self.last_copied_index, index));
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

    #[cfg(feature = "segment")]
    #[test]
    fn segment() {
        let uw = UnicodeWidth::new();
        // Test combining characters "a\u{301}" (grapheme cluster of width 1).
        // If max_width is 1: we consume 'a' and '\u{301}' successfully because
        // the first grapheme boundary is at 0, next is at 3.
        let mut iter = WidthIterator::new(&uw, "a\u{301}b");
        iter.set_max_width(1);
        assert_eq!(iter.next(), Some(('a', 1)));
        assert_eq!(iter.next(), Some(('\u{301}', 0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.width(), 1);
        assert_eq!(iter.input_end_index, Some(3)); // ends before 'b'
    }

    #[cfg(feature = "segment")]
    #[test]
    fn segment_max0() {
        let uw = UnicodeWidth::new();
        // If max_width is 0: it doesn't fit, and we stop at 0 (empty).
        let mut iter = WidthIterator::new(&uw, "a\u{301}b");
        iter.set_max_width(0);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.width(), 0);
        assert_eq!(iter.input_end_index, Some(0));

        // If max_width is 0 but include_at_least_one is true:
        // We must include the first grapheme cluster completely.
        let mut iter = WidthIterator::new(&uw, "a\u{301}b");
        iter.set_max_width(0);
        iter.set_include_at_least_one(true);
        assert_eq!(iter.next(), Some(('a', 1)));
        assert_eq!(iter.next(), Some(('\u{301}', 0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.width(), 1);
        assert_eq!(iter.input_end_index, Some(3));
    }

    #[cfg(feature = "segment")]
    #[test]
    fn segment_regional_indicators() {
        let uw = UnicodeWidth::new();
        // Test "\u{1F1FA}\u{1F1F8}b" (regional indicators, each having width 1,
        // total width 2).
        // If max_width is 2: both fit.
        let mut iter = WidthIterator::new(&uw, "\u{1F1FA}\u{1F1F8}b");
        iter.set_max_width(2);
        assert_eq!(iter.next(), Some(('\u{1F1FA}', 1)));
        assert_eq!(iter.next(), Some(('\u{1F1F8}', 1)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.width(), 2);
        assert_eq!(iter.input_end_index, Some(8));

        // If max_width is 1: since regional indicator flag "\u{1F1FA}\u{1F1F8}"
        // requires 2 width, and index 4 (between '\u{1F1FA}' and '\u{1F1F8}')
        // is not a grapheme boundary, it must stop at 0 on the second next()
        // call.
        let mut iter = WidthIterator::new(&uw, "\u{1F1FA}\u{1F1F8}b");
        iter.set_max_width(1);
        assert_eq!(iter.next(), Some(('\u{1F1FA}', 1)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.width(), 0);
        assert_eq!(iter.input_end_index, Some(0));

        // If max_width is 1 and include_at_least_one is true:
        // We must include the first grapheme cluster "\u{1F1FA}\u{1F1F8}"
        // entirely even if it exceeds max_width.
        let mut iter = WidthIterator::new(&uw, "\u{1F1FA}\u{1F1F8}b");
        iter.set_max_width(1);
        iter.set_include_at_least_one(true);
        assert_eq!(iter.next(), Some(('\u{1F1FA}', 1)));
        assert_eq!(iter.next(), Some(('\u{1F1F8}', 1)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.width(), 2);
        assert_eq!(iter.input_end_index, Some(8));
    }
}
