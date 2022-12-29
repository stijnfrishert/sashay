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
