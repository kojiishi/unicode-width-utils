use crate::UnicodeWidth;
use std::{borrow::Cow, str::CharIndices};

#[derive(Debug)]
pub(crate) struct WidthIterator<'a, 'b> {
    uw: &'a UnicodeWidth,
    input_str: &'b str,
    input_chars: CharIndices<'b>,
    position: usize,
    max_width: usize,
    input_end_index: Option<usize>,
    output: Option<String>,
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
            position: 0,
            max_width: usize::MAX,
            input_end_index: None,
            output: None,
        }
    }

    #[inline]
    pub(crate) fn set_max_width(&mut self, max_width: usize) {
        self.max_width = max_width;
    }

    pub(crate) fn consume_all(&mut self) {
        for _ in self {}
    }
}

impl<'a, 'b> Iterator for WidthIterator<'a, 'b> {
    type Item = (char, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let Some((index, ch)) = self.input_chars.next() else {
            self.input_end_index = Some(self.input_str.len());
            return None;
        };
        let ch_width = if let Some(ch_width) = self.uw.char_opt(ch) {
            ch_width
        } else if ch == '\t' && self.uw.tab_size > 0 {
            let tab_size = self.uw.tab_size as usize;
            if self.output.is_none() && self.uw.should_expand_tab {
                let mut output = String::with_capacity(self.input_str.len() + tab_size * 4);
                output.push_str(&self.input_str[..index]);
                self.output = Some(output);
            }
            tab_size - (self.position % tab_size)
        } else {
            0
        };
        let new_position = self.position + ch_width;
        if new_position > self.max_width {
            self.input_end_index = Some(index);
            return None;
        }
        self.position = new_position;
        if let Some(ref mut output) = self.output {
            if ch == '\t' {
                for _ in 0..ch_width {
                    output.push(' ');
                }
            } else {
                output.push(ch);
            }
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
        assert_eq!(iter.position, 1);
        assert_eq!(iter.next(), Some(('\t', 3)));
        assert_eq!(iter.position, 4);
        assert_eq!(iter.next(), Some(('B', 1)));
        assert_eq!(iter.position, 5);
        assert_eq!(iter.next(), None);
    }
}
