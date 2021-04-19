use std::fmt::Debug;
use std::ops::{Add, AddAssign, Sub, SubAssign};

pub trait AsIndex {
    const MAX: usize;

    fn as_index(&self) -> usize;
}

pub trait SuffixIndex:
    AddAssign
    + Add<Output = Self>
    + SubAssign
    + Sub<Output = Self>
    + Ord
    + Sized
    + Copy
    + PartialEq
    + Debug
    + AsIndex
{
    fn from_index(value: usize) -> Self;
}

impl AsIndex for usize {
    const MAX: Self = usize::MAX;

    #[inline(always)]
    fn as_index(&self) -> usize {
        *self
    }
}

impl SuffixIndex for usize {
    #[inline(always)]
    fn from_index(value: usize) -> Self {
        value as Self
    }
}

impl AsIndex for u8 {
    const MAX: usize = u8::MAX as usize;

    #[inline(always)]
    fn as_index(&self) -> usize {
        *self as usize
    }
}

impl SuffixIndex for u8 {
    #[inline(always)]
    fn from_index(value: usize) -> Self {
        debug_assert!(value <= Self::MAX as usize);
        value as Self
    }
}

impl AsIndex for u32 {
    const MAX: usize = u32::MAX as usize;

    #[inline(always)]
    fn as_index(&self) -> usize {
        *self as usize
    }
}

impl SuffixIndex for u32 {
    #[inline(always)]
    fn from_index(value: usize) -> Self {
        debug_assert!(value <= Self::MAX as usize);
        value as Self
    }
}

#[cfg(target_pointer_width = "64")]
impl AsIndex for u64 {
    const MAX: usize = u64::MAX as usize;

    #[inline(always)]
    fn as_index(&self) -> usize {
        *self as usize
    }
}

impl SuffixIndex for u64 {
    #[inline(always)]
    fn from_index(value: usize) -> Self {
        value as Self
    }
}
