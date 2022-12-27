use std::ops::Range;

pub fn sub_range(outer: Range<usize>, inner: Range<usize>) -> Range<usize> {
    let start = (outer.start + inner.start).clamp(outer.start, outer.end);
    let end = (outer.start + inner.end).clamp(outer.start, outer.end);
    start..end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sub_ranges() {
        assert_eq!(sub_range(5..10, 0..2), 5..7);
        assert_eq!(sub_range(5..10, 3..5), 8..10);
        assert_eq!(sub_range(5..10, 0..7), 5..10);
        assert_eq!(sub_range(5..10, 3..7), 8..10);
        assert!(sub_range(5..10, 5..7).is_empty());
    }
}
