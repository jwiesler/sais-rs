use std::cmp::Ordering;
use std::mem::replace;

use crate::suffix_index::{AsIndex, SuffixIndex};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Type {
    L,
    S,
}

impl Default for Type {
    fn default() -> Self {
        Self::L
    }
}

fn classify<C: Ord>(text: &[C], types: &mut [Type]) {
    debug_assert_eq!(types.len(), text.len());
    debug_assert_ne!(text.len(), 0);

    *types.last_mut().unwrap() = Type::L;
    classify_sub_slice(text, types);
}

/// Assumes
/// - `types[len - 1]` is already set
fn classify_sub_slice<C: Ord>(text: &[C], types: &mut [Type]) {
    use Type::*;

    for i in (0..types.len() - 1).rev() {
        let r = match text[i].cmp(&text[i + 1]) {
            Ordering::Less => S,
            Ordering::Greater => L,
            Ordering::Equal => types[i + 1],
        };

        types[i] = r;
    }
}

fn buckets_count<C: AsIndex, I: SuffixIndex>(text: &[C], buckets: &mut [I]) {
    if cfg!(debug_assertions) {
        assert!(buckets.iter().all(|v| v.as_index() == 0));
    }

    for bucket in text.iter().map(AsIndex::as_index) {
        buckets[bucket] += I::from_index(1);
    }
}

fn bucket_ends<I: SuffixIndex>(buckets: &mut [I]) {
    let mut sum = I::from_index(0);
    for value in buckets.iter_mut() {
        *value += sum;
        sum = *value;
    }
}

fn bucket_starts<I: SuffixIndex>(buckets: &mut [I]) {
    let mut sum = I::from_index(0);
    for value in buckets.iter_mut() {
        let value = replace(value, sum);
        sum += value;
    }
}

struct Buckets<'a, C, I> {
    buckets: &'a mut [I],
    text: &'a [C],
}

impl<'a, C: AsIndex, I: SuffixIndex> Buckets<'a, C, I> {
    fn make_starts(text: &'a [C], buckets: &'a mut [I]) -> Self {
        buckets_count(text, buckets);
        bucket_starts(buckets);
        Self { buckets, text }
    }

    fn make_ends(text: &'a [C], buckets: &'a mut [I]) -> Self {
        buckets_count(text, buckets);
        bucket_ends(buckets);
        Self { buckets, text }
    }

    fn suffix_bucket_next(&mut self, suffix: I) -> I {
        let bucket = self.text[suffix.as_index()].as_index();
        self.next(bucket)
    }

    fn suffix_bucket_next_reverse(&mut self, suffix: I) -> I {
        let bucket = self.text[suffix.as_index()].as_index();
        self.next_reverse(bucket)
    }
}

impl<'a, C, I: SuffixIndex> Buckets<'a, C, I> {
    fn next(&mut self, bucket: usize) -> I {
        let bucket_start = &mut self.buckets[bucket];
        let index = *bucket_start;
        bucket_start.add_assign(I::from_index(1));
        index
    }

    fn next_reverse(&mut self, bucket: usize) -> I {
        let bucket_start = &mut self.buckets[bucket];
        bucket_start.sub_assign(I::from_index(1));
        *bucket_start
    }

    fn into_cleared(self) -> &'a mut [I] {
        let buckets = self.buckets;
        buckets.fill(I::from_index(0));
        buckets
    }
}

fn is_lms<I: SuffixIndex>(suffix: I, types: &[Type]) -> bool {
    use Type::*;
    debug_assert_ne!(suffix, I::from_index(0));
    matches!(
        (types[suffix.as_index() - 1], types[suffix.as_index()]),
        (L, S)
    )
}

/// Assumes:
/// - text has a lms character at index
fn lms_substring<'a, C>(index: usize, text: &'a [C], types: &[Type]) -> &'a [C] {
    debug_assert!(index < text.len());
    debug_assert_ne!(index, 0);

    for i in index + 1..text.len() {
        if is_lms(i, types) {
            return &text[index..=i];
        }
    }
    &text[index..]
}

fn lms_substrings_eq<C: Eq>(
    left: &[C],
    left_types: &[Type],
    right: &[C],
    right_types: &[Type],
) -> bool {
    debug_assert_eq!(left_types.len(), left.len());
    debug_assert_eq!(right_types.len(), right.len());
    if left.len() != right.len() {
        false
    } else {
        for ((l, lt), (r, rt)) in left
            .iter()
            .zip(left_types)
            .zip(right.iter().zip(right_types.iter()))
        {
            if l != r || lt != rt {
                return false;
            }
        }
        true
    }
}

/// Moves all values matching the predicate to the front of the slice
/// The remaining values are unspecified
fn retain<'a, T: Copy, P: FnMut(&T) -> bool>(
    values: &mut [T],
    mut predicate: P,
) -> (&mut [T], &mut [T]) {
    let mut write_offset = 0;
    for i in 0..values.len() {
        let value = &values[i];
        if predicate(value) {
            values[write_offset] = *value;
            write_offset += 1;
        }
    }

    values.split_at_mut(write_offset)
}

fn induce_ls<C: AsIndex, I: SuffixIndex>(
    text: &[C],
    types: &[Type],
    buckets: &mut [I],
    suffixes: &mut [I],
) {
    use Type::*;

    // Step 2
    let mut buckets = Buckets::make_starts(text, buckets);

    let last = I::from_index(suffixes.len() - 1);
    if let L = types[last.as_index()] {
        let index = buckets.suffix_bucket_next(last).as_index();
        suffixes[index] = last;
    }
    for i in 0..suffixes.len() {
        let suffix = suffixes[i];
        if suffix != I::from_index(I::MAX) && suffix != I::from_index(0) {
            let previous_suffix: I = suffix - I::from_index(1);
            if let L = types[previous_suffix.as_index()] {
                // Push previous_suffix to the front of its bucket
                let index = buckets.suffix_bucket_next(previous_suffix).as_index();
                suffixes[index] = previous_suffix;
            }
        }
    }

    // Step 3
    let buckets = buckets.into_cleared();
    let mut buckets = Buckets::make_ends(text, buckets);

    for i in (0..suffixes.len()).rev() {
        let suffix = suffixes[i];
        if suffix != I::from_index(I::MAX) && suffix != I::from_index(0) {
            let previous_suffix: I = suffix - I::from_index(1);
            if let S = types[previous_suffix.as_index()] {
                // Push previous_suffix to the back of its bucket
                let index = buckets
                    .suffix_bucket_next_reverse(previous_suffix)
                    .as_index();
                suffixes[index] = previous_suffix;
            }
        }
    }
    buckets.into_cleared();
}

fn induce<'a, C: AsIndex + Eq, I: SuffixIndex>(
    text: &[C],
    types: &[Type],
    suffixes: &'a mut [I],
    buckets: &mut [I],
) -> Option<Reduced<'a, I>> {
    debug_assert_ne!(types.len(), 0);
    suffixes.fill(I::from_index(0));

    let mut buckets = Buckets::make_ends(text, buckets);
    let mut lms_count = 0;
    let mut last_lms = None;
    for suffix in 1..types.len() {
        let suffix = I::from_index(suffix);
        if is_lms(suffix, types) {
            let index = buckets.suffix_bucket_next_reverse(suffix).as_index();
            suffixes[index] = suffix;
            lms_count += 1;
            last_lms = Some(suffix);
        }
    }
    let buckets = buckets.into_cleared();

    if lms_count > 1 {
        induce_ls(text, types, buckets, suffixes);
        let reduce = reduce(text, types, suffixes);
        debug_assert_eq!(reduce.lms_suffixes_sorted.len(), lms_count);
        Some(reduce)
    } else if lms_count == 1 {
        let lms = last_lms.unwrap();
        let (lms_suffixes_sorted, rest) = suffixes.split_at_mut(1);
        lms_suffixes_sorted[0] = lms;
        let (reduced_str, _) = rest.split_at_mut(1);
        reduced_str[0] = I::from_index(0);
        Some(Reduced {
            lms_suffixes_sorted,
            reduced_str,
            max_order: 0,
        })
    } else {
        None
    }
}

#[derive(Debug)]
struct Reduced<'a, I> {
    /// len >= 1
    lms_suffixes_sorted: &'a mut [I],
    /// len >= 1
    reduced_str: &'a mut [I],
    max_order: usize,
}

/// Assumes:
/// - text.len() > 2
/// - >= 1 lms substrings
/// - suffixes contains the sorted lms substrings
fn reduce<'a, C: AsIndex + Eq, I: SuffixIndex>(
    text: &[C],
    types: &[Type],
    suffixes: &'a mut [I],
) -> Reduced<'a, I> {
    // There is at most 1 lms every two characters
    // - len/2 lms suffixes
    // - each lms has a unique index in [0..len/2]: i/2

    // Compact all the sorted lms suffixes to the front of the vector
    let (lms_suffixes_sorted, rest) = retain(suffixes, |&suffix| {
        suffix != I::from_index(0) && is_lms(suffix, types)
    });
    debug_assert!(!lms_suffixes_sorted.is_empty());
    debug_assert!(lms_suffixes_sorted.len() <= text.len() / 2);

    let (reduced_str, max_order) = {
        rest.fill(I::from_index(I::MAX));

        let mut iter = lms_suffixes_sorted.iter();
        let (mut last_str, mut last_types) = {
            let first_suffix = iter.next().unwrap().as_index();
            rest[first_suffix / 2] = I::from_index(0);
            let str = lms_substring(first_suffix, text, types);
            let types = &types[first_suffix..first_suffix + str.len()];
            (str, types)
        };

        let mut order = 0;
        for suffix in iter {
            let suffix = suffix.as_index();
            let sub_str = lms_substring(suffix, text, types);
            let types = &types[suffix..suffix + sub_str.len()];
            if !lms_substrings_eq(last_str, last_types, sub_str, types) {
                order += 1;
            }
            rest[suffix / 2] = I::from_index(order);
            last_str = sub_str;
            last_types = types;
        }

        (
            retain(rest, |&order| order != I::from_index(I::MAX)).0,
            order,
        )
    };

    Reduced {
        lms_suffixes_sorted,
        reduced_str,
        max_order,
    }
}

fn induced_sort<C: AsIndex + Ord, I: SuffixIndex>(
    text: &[C],
    suffix_array: &mut [I],
    types: &mut [Type],
    buckets: &mut Vec<I>,
) {
    debug_assert_eq!(text.len(), suffix_array.len());
    if cfg!(debug_assertions) {
        for c in text {
            assert!(c.as_index() < buckets.len());
        }
    }

    classify(text, types);
    let reduced = induce(text, types, suffix_array, buckets);
    if let Some(reduced) = reduced {
        let Reduced {
            lms_suffixes_sorted,
            reduced_str,
            max_order,
        } = reduced;
        debug_assert!(max_order <= reduced_str.len());
        debug_assert_eq!(lms_suffixes_sorted.len(), reduced_str.len());
        let lms_count = lms_suffixes_sorted.len();
        if max_order < reduced_str.len() - 1 {
            // let buckets = &mut buckets[..=max_order];
            let suffix_array = lms_suffixes_sorted;
            let required_len = max_order + 1;
            let old_len = buckets.len();
            buckets.resize(required_len, I::from_index(0));

            induced_sort(
                reduced_str,
                suffix_array,
                &mut types[..suffix_array.len()],
                buckets,
            );

            // restore
            classify_sub_slice(text, &mut types[..suffix_array.len() + 1]);
            buckets.resize(old_len, I::from_index(0));
            buckets.fill(I::from_index(0));

            // Convert the lexical names to suffix indices, lookup their order, write to lms_suffixes_sorted
            let suffix_indices = reduced_str;
            let mut suffix_indices_offset = 0;
            for suffix in (1..types.len()).map(I::from_index) {
                if is_lms(suffix, types) {
                    suffix_indices[suffix_indices_offset] = suffix;
                    suffix_indices_offset += 1;
                }
            }

            for i in 0..lms_count {
                suffix_array[i] = suffix_indices[suffix_array[i].as_index()];
            }
        } else {
            // reduced_str is unique => this is the order
        }

        // lms_suffixes_sorted now contains all lms suffixes in the correct order
        suffix_array[lms_count..].fill(I::from_index(I::MAX));

        // put LMS in their buckets
        let mut buckets = Buckets::make_ends(text, buckets);

        // Right to left fill lms suffixes in their buckets
        // This does not overwrite the sorted lms indices
        for i in (0..lms_count).rev() {
            let suffix = suffix_array[i];
            suffix_array[i] = I::from_index(I::MAX);
            let index = buckets.suffix_bucket_next_reverse(suffix).as_index();
            suffix_array[index] = suffix;
        }
        buckets.into_cleared();
    }

    induce_ls(text, types, buckets, suffix_array);
}

pub fn sort<I: SuffixIndex, C: AsIndex + Ord>(
    text: &[C],
    suffix_array: &mut [I],
    types: &mut [Type],
    buckets: &mut Vec<I>,
) {
    assert_eq!(text.len(), suffix_array.len());
    assert_eq!(text.len(), types.len());
    assert!(buckets.len() - 1 >= C::MAX);
    induced_sort(text, suffix_array, types, buckets);
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::Read;
    use std::time::SystemTime;

    use super::*;

    fn is_sorted<I: AsIndex>(indices: &[I], text: &[u8]) -> Option<usize> {
        let compare = |a: &&I, b: &&I| (&text[a.as_index()..]).cmp(&text[b.as_index()..]);
        indices
            .windows(2)
            .enumerate()
            .find_map(|(index, w)| (compare(&&w[0], &&w[1]) == Ordering::Greater).then(|| index))
    }

    #[test]
    fn test_sort() {
        const TEXT: &str = "And now map the suffix indices from the reduced text to suffix";
        let mut types = [Type::S; TEXT.len()];
        classify(TEXT.as_bytes(), &mut types);
        for i in 0..TEXT.len() {
            print!("{} ", i % 10)
        }
        println!();

        for i in TEXT.chars() {
            print!("{} ", i);
        }
        println!();

        for i in 0..TEXT.len() {
            print!("{:?} ", types[i])
        }
        println!();

        print!("  ");
        for i in 1..TEXT.len() {
            if is_lms(i, &types) {
                print!("* ");
            } else {
                print!("  ");
            }
        }
        println!();

        let mut buckets = vec![0u32; 256];
        let mut output = [0u32; TEXT.len()];
        induced_sort(TEXT.as_bytes(), &mut output, &mut types, &mut buckets);

        for &i in output.iter() {
            println!("{} {:?}", i, &TEXT[i as usize..])
        }

        assert_eq!(is_sorted(&output, TEXT.as_bytes()), None)
    }

    #[test]
    fn test_sort_file() {
        let mut text = Vec::new();
        File::open("gauntlet_corpus/abac")
            .unwrap()
            .read_to_end(&mut text)
            .unwrap();
        let mut indices = vec![0u32; text.len()];
        let time = SystemTime::now();
        let mut buckets = vec![0u32; 256];
        let mut types = vec![Type::L; text.len()];
        induced_sort(&text, &mut indices, &mut types, &mut buckets);
        println!("{:?}", time.elapsed().unwrap());

        assert_eq!(is_sorted(&indices, &text), None);
    }
}
