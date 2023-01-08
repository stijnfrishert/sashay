use core::ops::{Bound, Range, RangeBounds};

pub fn constrain_range<R>(len: usize, range: R) -> Range<usize>
where
    R: RangeBounds<usize>,
{
    let start = match range.start_bound() {
        Bound::Included(start) => *start,
        Bound::Excluded(start) => *start + 1,
        Bound::Unbounded => 0,
    }
    .min(len);

    let end = match range.end_bound() {
        Bound::Included(end) => *end + 1,
        Bound::Excluded(end) => *end,
        Bound::Unbounded => len,
    }
    .min(len);

    start..end
}

#[test]
fn range_types() {
    assert_eq!(constrain_range(5, 0..2), 0..2); // Range
    assert_eq!(constrain_range(5, 3..7), 3..5);
    assert_eq!(constrain_range(5, ..3), 0..3);
    assert_eq!(constrain_range(5, 7..), 5..5); // RangeFrom
    assert_eq!(constrain_range(5, ..), 0..5); // RangeFull
    assert_eq!(constrain_range(5, 1..=2), 1..3); // RangeInclusive
    assert_eq!(constrain_range(5, ..4), 0..4); // RangeTo
    assert_eq!(constrain_range(5, ..=2), 0..3); // RangeToInclusive
}
