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
        Self {
            uw,
            source: WidthSource::new(
                input,
                #[cfg(feature = "ansi")]
                uw.is_ansi,
            ),
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
        let Some((index, ch)) = self.source.next() else {
            self.set_input_end_index(self.source.len());
            return None;
        };
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
}
