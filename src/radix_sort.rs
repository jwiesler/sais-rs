use crate::suffix_index::SuffixIndex;

const BUCKETS: usize = 0x100;

fn exclusive_sum(values: &mut [usize]) {
    let mut sum = 0;
    for value in values {
        let value = std::mem::replace(value, sum);
        sum += value;
    }
}

/// # Safety
/// - the caller must ensure that all `indices` are in range of `text`
/// - buckets contain the count of the elements in this bucket
unsafe fn move_elements_in_place<T: SuffixIndex>(
    indices: &mut [T],
    text: &[u8],
    buckets: &mut [usize; BUCKETS],
) {
    exclusive_sum(buckets);
    // buckets now contain the next index that should be occupied by an element of the bucket

    let mut i = 0;
    while i < indices.len() {
        debug_assert!(i <= indices.len());
        let suffix = indices.get_unchecked(i).as_index();
        debug_assert!(suffix <= text.len());
        let bucket = *text.get_unchecked(suffix) as usize;
        let offset = buckets.get_unchecked_mut(bucket);

        // Swap only with elements with a smaller index than the current element
        if i < *offset {
            i += 1;
            continue;
        }

        if *offset == i {
            i += 1;
        } else {
            indices.swap(i, *offset);
        }

        *offset += 1;
    }
}

/// # Safety
/// Invariants
/// - >= 2 indices
/// - `indices` contains only valid unique indices
/// - there is at most one `index == text.len()`
unsafe fn suffix_sort<T: SuffixIndex>(mut indices: &mut [T], mut text: &[u8]) {
    // `indices` contains only unique indices is always maintained
    // since we only remove or swap indices

    let mut buckets = [0usize; BUCKETS];
    // Invariants:
    // - buckets is clear
    // - remaining function invariants
    loop {
        if text.is_empty() {
            return;
        }

        let mut empty = None;
        for (i, index) in indices.iter().enumerate() {
            let index = index.as_index();
            if index == text.len() {
                debug_assert!(empty.is_none());
                empty = Some(i);
            } else {
                let bucket = text[index] as usize;
                buckets[bucket] += 1;
            }
        }

        // after this, there is no empty suffix contained in `indices`, indices.len() is >= 1
        if let Some(empty) = empty {
            indices.swap(0, empty);
            // Safety: indices.len() >= 2
            debug_assert!(indices.len() >= 2);
            indices = indices.get_unchecked_mut(1..);
        }

        // Safety: indices.len() is >= 1
        debug_assert!(indices.len() >= 1);

        let first_bucket_index = *text.get_unchecked(indices.get_unchecked(0).as_index()) as usize;
        if *buckets.get_unchecked(first_bucket_index) == indices.len() {
            if indices.len() >= 2 {
                text = &text[1..];
            } else {
                return;
            }
        } else {
            break;
        }

        buckets[first_bucket_index] = 0;
    }

    // Safety:
    // - all indices are valid for `text` (we only removed at most one)
    // - buckets contain element count
    move_elements_in_place(indices, text, &mut buckets);
    // bucket contains array offset of one plus the last item in the bucket

    let mut last_end = 0;
    for &bucket_end in buckets.iter() {
        if bucket_end - last_end >= 2 {
            // Safety: last_end <= bucket_end and bucket_end <= indices.len()
            let indices = indices.get_unchecked_mut(last_end..bucket_end);
            // Safety: text was non empty before
            let text = text.get_unchecked(1..);

            // Safety suffix_sort:
            // - >= 2 indices
            // - no empty suffix, next character is valid for all `indices` (and all subsets)
            // - at most one empty suffix is created since all indices were unique
            suffix_sort(indices, text);
        }

        last_end = bucket_end;
    }
}

/// # Safety
/// the caller must ensure `indices` contains all valid indices exactly once
pub unsafe fn sort<T: SuffixIndex>(indices: &mut [T], text: &[u8]) {
    if indices.len() <= 1 {
        return;
    }
    suffix_sort(indices, text);
}

pub fn make_suffix_array<T: SuffixIndex>(text: &[u8]) -> Vec<T> {
    assert!(text.len() < T::MAX);
    let mut indices = (0..text.len())
        .map(|i| T::from_index(i))
        .collect::<Vec<_>>();
    unsafe {
        sort(&mut indices, text);
    }
    indices
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;
    use std::fs::File;
    use std::io::Read;

    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_sum() {
        let mut v = [];
        exclusive_sum(&mut v);
        assert_eq!(&v, &[0usize; 0]);

        let mut v = [1];
        exclusive_sum(&mut v);
        assert_eq!(&v, &[0]);

        let mut v = [0, 1];
        exclusive_sum(&mut v);
        assert_eq!(&v, &[0, 0]);

        let mut v = [1, 2];
        exclusive_sum(&mut v);
        assert_eq!(&v, &[0, 1]);
    }

    fn is_sorted(indices: &[usize], text: &[u8]) -> Option<usize> {
        let compare = |&a, &b| (&text[a..]).cmp(&text[b..]);
        indices
            .windows(2)
            .enumerate()
            .find_map(|(index, w)| (compare(&&w[0], &&w[1]) == Ordering::Greater).then(|| index))
    }

    #[test]
    fn test_sort() {
        let text = "A\0BB\0CCC\0DD\0E";
        let mut indices = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
        unsafe {
            sort(&mut indices, text.as_bytes());
        }

        assert_eq!(is_sorted(&indices, text.as_bytes()), None);
    }

    #[test]
    fn test_sort_repeating() {
        let text = "AAAAAAAAAAAAA";
        let mut indices = (0..text.len()).collect::<Vec<_>>();
        unsafe {
            sort(&mut indices, text.as_bytes());
        }
    }

    #[test]
    fn test_sort_file() {
        let mut text = String::new();
        File::open("gauntlet_corpus/paper5x80")
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        let mut indices = (0..=text.len()).collect::<Vec<_>>();
        let time = SystemTime::now();
        unsafe {
            sort(&mut indices, text.as_bytes());
        }
        println!("{:?}", time.elapsed().unwrap());

        // assert_eq!(is_sorted(&indices, text.as_bytes()), None);
    }
}
