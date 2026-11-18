use std::collections::HashSet;

use linqguru::LinqExt;
use proptest::prelude::*;

fn small_vec() -> impl Strategy<Value = Vec<i32>> {
    proptest::collection::vec(-8i32..=8, 0..40)
}

fn keyed_vec() -> impl Strategy<Value = Vec<(i32, u16)>> {
    proptest::collection::vec((-4i32..=4, 0u16..1000), 0..40)
}

fn first_occurrences(input: &[i32]) -> Vec<i32> {
    let mut seen = HashSet::new();
    input.iter().filter(|v| seen.insert(**v)).copied().collect()
}

proptest! {
    #[test]
    fn distinct_is_first_occurrence_order(input in small_vec()) {
        let out: Vec<i32> = input.clone().into_iter().distinct().to_list();
        prop_assert_eq!(out, first_occurrences(&input));
    }

    #[test]
    fn except_excludes_and_deduplicates(input in small_vec(), excluded in small_vec()) {
        let out: Vec<i32> = input.clone().into_iter().except(excluded.clone()).to_list();
        let excluded: HashSet<i32> = excluded.into_iter().collect();
        let mut seen = HashSet::new();
        let mut expected = Vec::new();
        for v in input {
            if !excluded.contains(&v) && seen.insert(v) {
                expected.push(v);
            }
        }
        prop_assert_eq!(out, expected);
    }

    #[test]
    fn intersect_is_set_intersection_in_source_order(input in small_vec(), other in small_vec()) {
        let out: Vec<i32> = input.clone().into_iter().intersect(other.clone());
        let other: HashSet<i32> = other.into_iter().collect();
        let mut seen = HashSet::new();
        let mut expected = Vec::new();
        for v in input {
            if other.contains(&v) && seen.insert(v) {
                expected.push(v);
            }
        }
        prop_assert_eq!(out, expected);
    }

    #[test]
    fn union_equals_distinct_concat(input in small_vec(), other in small_vec()) {
        let out: Vec<i32> = input.clone().into_iter().union(other.clone()).to_list();
        let mut chained = input;
        chained.extend(other);
        prop_assert_eq!(out, first_occurrences(&chained));
    }

    #[test]
    fn order_by_matches_stable_sort(input in keyed_vec()) {
        let out = input.clone().into_iter().order_by(|(k, _)| *k);
        let mut expected = input.clone();
        expected.sort_by_key(|(k, _)| *k);
        prop_assert_eq!(out, expected);
    }

    #[test]
    fn order_by_descending_matches_stable_sort(input in keyed_vec()) {
        let out = input.clone().into_iter().order_by_descending(|(k, _)| *k);
        let mut expected = input.clone();
        expected.sort_by_key(|(k, _)| std::cmp::Reverse(*k));
        prop_assert_eq!(out, expected);
    }

    #[test]
    fn chunk_linq_preserves_items_and_covers_all(input in small_vec(), size in 0usize..6) {
        let chunks = input.clone().into_iter().chunk_linq(size);
        prop_assert_eq!(chunks.iter().flatten().copied().collect::<Vec<_>>(), input);
        let effective = size.max(1);
        for (i, chunk) in chunks.iter().enumerate() {
            let is_last = i + 1 == chunks.len();
            prop_assert!(!chunk.is_empty() && chunk.len() <= effective);
            if !is_last {
                prop_assert_eq!(chunk.len(), effective);
            }
        }
    }

    #[test]
    fn skip_take_is_slice(input in small_vec(), skip in 0usize..45, take in 0usize..45) {
        let out: Vec<i32> = input
            .clone()
            .into_iter()
            .skip_linq(skip)
            .take_linq(take)
            .to_list();
        prop_assert_eq!(out, input.into_iter().skip(skip).take(take).collect::<Vec<_>>());
    }

    #[test]
    fn group_by_linq_groups_in_first_occurrence_order(input in small_vec()) {
        let grouped = input
            .clone()
            .into_iter()
            .group_by_linq(|n| n.rem_euclid(3), |n| *n);
        let mut expected_keys = Vec::new();
        let mut expected_values: std::collections::HashMap<i32, Vec<i32>> = std::collections::HashMap::new();
        for v in input {
            let k = v.rem_euclid(3);
            match expected_values.entry(k) {
                std::collections::hash_map::Entry::Vacant(e) => {
                    e.insert(vec![v]);
                    expected_keys.push(k);
                }
                std::collections::hash_map::Entry::Occupied(mut e) => {
                    e.get_mut().push(v);
                }
            }
        }
        let got_keys: Vec<i32> = grouped.iter().map(|(k, _)| *k).collect();
        prop_assert_eq!(got_keys, expected_keys);
        for (k, values) in grouped {
            prop_assert_eq!(values, expected_values[&k].clone());
        }
    }
}
