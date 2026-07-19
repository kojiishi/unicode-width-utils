use crate::UnicodeWidth;
use crate::WidthIterator;
use std::borrow::Cow;

/// An iterator over chunks of a string based on display width.
///
/// It supports all configurations in [`UnicodeWidth`] such as tab expansion.
///
/// Unlike [`UnicodeWidth::truncate()`],
/// each line is guaranteed to have at least one character,
/// even if it is wider than `max_width`.
///
/// See also [`UnicodeWidth::lines()`].
///
/// # Examples
/// ```
/// use unicode_width_utils::UnicodeWidth;
///
/// let uw = UnicodeWidth::new();
/// assert_eq!(
///     uw.lines("12345678", 3).collect::<Vec<_>>(),
///     vec!["123", "456", "78"]
/// );
/// ```
#[derive(Debug)]
pub struct LineIterator<'a, 'b> {
    uw: &'a UnicodeWidth,
    input: &'b str,
    max_width: usize,
}

impl<'a, 'b> LineIterator<'a, 'b> {
    /// Create a new `LineIterator` from a string slice.
    pub(crate) fn new(uw: &'a UnicodeWidth, input: &'b str, max_width: usize) -> Self {
        Self {
            uw,
            input,
            max_width,
        }
    }

    /// Return the rest of the string.
    ///
    /// # Examples
    /// ```
    /// use unicode_width_utils::UnicodeWidth;
    ///
    /// let uw = UnicodeWidth::new();
    /// let mut iter = uw.lines("12345", 3);
    /// assert_eq!(iter.rest(), "12345");
    /// assert_eq!(iter.next().unwrap(), "123");
    /// assert_eq!(iter.rest(), "45");
    /// assert_eq!(iter.next().unwrap(), "45");
    /// assert_eq!(iter.rest(), "");
    /// ```
    pub fn rest(&self) -> &'b str {
        self.input
    }
}

impl<'a, 'b> Iterator for LineIterator<'a, 'b> {
    type Item = Cow<'b, str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            return None;
        }
        let mut iter = WidthIterator::new(self.uw, self.input);
        iter.set_max_width(self.max_width);
        iter.set_include_at_least_one(true);
        iter.consume_all();
        let end_index = iter.input_end_index.unwrap();
        self.input = &self.input[end_index..];
        Some(iter.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_tabs() {
        let uw = UnicodeWidth::new();
        let input = "hello world";
        let mut iter = LineIterator::new(&uw, input, 5);
        assert_eq!(iter.rest(), "hello world");
        assert_eq!(iter.next(), Some(Cow::Borrowed("hello")));
        assert_eq!(iter.rest(), " world");
        assert_eq!(iter.next(), Some(Cow::Borrowed(" worl")));
        assert_eq!(iter.rest(), "d");
        assert_eq!(iter.next(), Some(Cow::Borrowed("d")));
        assert_eq!(iter.rest(), "");
        assert_eq!(iter.next(), None);
        assert_eq!(iter.rest(), "");
    }

    #[test]
    fn expand_tabs() {
        let mut uw = UnicodeWidth::new();
        uw.set_tab_size(4);
        uw.set_expand_tab(true);
        let input = "A\tB\tCD";
        let mut iter = LineIterator::new(&uw, input, 5);
        assert_eq!(iter.next(), Some(Cow::Owned("A   B".to_string())));
        assert_eq!(iter.next(), Some(Cow::Owned("    C".to_string())));
        assert_eq!(iter.next(), Some(Cow::Borrowed("D")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn cjk_boundary() {
        let uw = UnicodeWidth::new();
        // CJK character "あ" has width 2.
        // "A" (width 1) + "あ" (width 2) = 3 columns.
        // With max_width = 2, "あ" does not fit on the first line and wraps to the second line.
        let mut iter = LineIterator::new(&uw, "Aあ", 2);
        assert_eq!(iter.next(), Some(Cow::Borrowed("A")));
        assert_eq!(iter.next(), Some(Cow::Borrowed("あ")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn at_least_one() {
        let uw = UnicodeWidth::new();
        // CJK character "あ" has width 2, but max_width is 1.
        let mut iter = LineIterator::new(&uw, "あA", 1);
        assert_eq!(iter.next(), Some(Cow::Borrowed("あ")));
        assert_eq!(iter.next(), Some(Cow::Borrowed("A")));
        assert_eq!(iter.next(), None);
    }
}
