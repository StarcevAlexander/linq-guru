pub trait LinqExt: Iterator + Sized {
    fn where_(
        self,
        mut predicate: impl FnMut(Self::Item) -> bool,
    ) -> impl Iterator<Item = Self::Item>
    where
        Self::Item: Clone,
    {
        self.filter(move |item| predicate(item.clone()))
    }

    fn where_with_index(
        self,
        mut predicate: impl FnMut(usize, Self::Item) -> bool,
    ) -> impl Iterator<Item = Self::Item>
    where
        Self::Item: Clone,
    {
        self.enumerate()
            .filter(move |(i, item)| predicate(*i, item.clone()))
            .map(|(_, item)| item)
    }

    fn select<B>(self, selector: impl FnMut(Self::Item) -> B) -> impl Iterator<Item = B> {
        self.map(selector)
    }

    fn select_with_index<B>(
        self,
        mut selector: impl FnMut(usize, Self::Item) -> B,
    ) -> impl Iterator<Item = B> {
        self.enumerate().map(move |(i, item)| selector(i, item))
    }

    fn order_by<K: Ord>(self, key: impl FnMut(&Self::Item) -> K) -> Vec<Self::Item>
    where
        Self::Item: Clone,
    {
        let mut items: Vec<Self::Item> = self.collect();
        items.sort_by_key(key);
        items
    }

    fn order_by_descending<K: Ord>(
        self,
        mut key: impl FnMut(&Self::Item) -> K,
    ) -> Vec<Self::Item>
    where
        Self::Item: Clone,
    {
        // Compute each key once, then sort by the key descending.
        // `sort_by` is stable, so ties keep their source order (as in LINQ).
        let mut keyed: Vec<(K, Self::Item)> = self.map(|item| (key(&item), item)).collect();
        keyed.sort_by(|a, b| b.0.cmp(&a.0));
        keyed.into_iter().map(|(_, item)| item).collect()
    }

    fn group_by_linq<K, B>(
        self,
        mut key_selector: impl FnMut(&Self::Item) -> K,
        mut value_selector: impl FnMut(&Self::Item) -> B,
    ) -> Vec<(K, Vec<B>)>
    where
        K: PartialEq,
        Self::Item: Clone,
    {
        let mut groups: Vec<(K, Vec<B>)> = Vec::new();
        for item in self {
            let key = key_selector(&item);
            match groups.iter_mut().find(|(k, _)| *k == key) {
                Some((_, values)) => values.push(value_selector(&item)),
                None => groups.push((key, vec![value_selector(&item)])),
            }
        }
        groups
    }

    fn distinct(self) -> impl Iterator<Item = Self::Item>
    where
        Self::Item: PartialEq + Clone,
    {
        distinct_iter(self.collect())
    }

    fn first_or_default(mut self, default: Self::Item) -> Self::Item {
        self.next().unwrap_or(default)
    }

    fn last_or_default(self, default: Self::Item) -> Self::Item {
        let mut last = default;
        for item in self {
            last = item;
        }
        last
    }

    fn count_where(self, mut predicate: impl FnMut(Self::Item) -> bool) -> usize
    where
        Self::Item: Clone,
    {
        self.filter(move |item| predicate(item.clone())).count()
    }

    fn aggregate<A>(self, seed: A, accumulator: impl FnMut(A, Self::Item) -> A) -> A {
        self.fold(seed, accumulator)
    }

    fn to_list(self) -> Vec<Self::Item> {
        self.collect()
    }

    fn except(self, other: impl IntoIterator<Item = Self::Item>) -> impl Iterator<Item = Self::Item>
    where
        Self::Item: PartialEq + Clone,
    {
        let excluded: Vec<Self::Item> = other.into_iter().collect();
        // LINQ semantics: each surviving value is yielded exactly once.
        let mut seen: Vec<Self::Item> = Vec::new();
        self.into_iter().filter_map(move |item| {
            if excluded.contains(&item) || seen.contains(&item) {
                None
            } else {
                seen.push(item.clone());
                Some(item)
            }
        })
    }

    fn intersect(self, other: impl IntoIterator<Item = Self::Item>) -> Vec<Self::Item>
    where
        Self::Item: PartialEq + Clone,
    {
        let other: Vec<Self::Item> = other.into_iter().collect();
        let mut out: Vec<Self::Item> = Vec::new();
        for item in self {
            if other.contains(&item) && !out.contains(&item) {
                out.push(item);
            }
        }
        out
    }

    fn union_all(
        self,
        other: impl IntoIterator<Item = Self::Item>,
    ) -> impl Iterator<Item = Self::Item> {
        self.into_iter().chain(other)
    }

    fn union(self, other: impl IntoIterator<Item = Self::Item>) -> impl Iterator<Item = Self::Item>
    where
        Self::Item: PartialEq + Clone,
    {
        distinct_iter(self.into_iter().chain(other).collect())
    }

    fn skip_linq(self, n: usize) -> impl Iterator<Item = Self::Item> {
        self.skip(n)
    }

    fn take_linq(self, n: usize) -> impl Iterator<Item = Self::Item> {
        self.take(n)
    }

    fn chunk_linq(self, size: usize) -> Vec<Vec<Self::Item>>
    where
        Self::Item: Clone,
    {
        let all = self.collect::<Vec<_>>();
        let size = size.max(1);
        all.chunks(size).map(|c| c.to_vec()).collect()
    }
}

impl<I: Iterator> LinqExt for I {}

fn distinct_iter<T: PartialEq + Clone>(items: Vec<T>) -> impl Iterator<Item = T> {
    let mut seen: Vec<T> = Vec::new();
    items.into_iter().filter(move |item| {
        if seen.contains(item) {
            false
        } else {
            seen.push(item.clone());
            true
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_pipeline() {
        let result = vec![1, 2, 3, 4, 5]
            .into_iter()
            .where_(|n| n % 2 == 0)
            .select(|n| n * 10)
            .to_list();
        assert_eq!(result, vec![20, 40]);
    }

    #[test]
    fn order_by_works() {
        let sorted = vec!["banana", "apple", "cherry"]
            .into_iter()
            .order_by(|s| s.to_string());
        assert_eq!(sorted, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn order_by_descending_works() {
        let sorted = vec![3, 1, 2].into_iter().order_by_descending(|n| *n);
        assert_eq!(sorted, vec![3, 2, 1]);
    }

    #[test]
    fn group_by_linq_preserves_order() {
        let grouped = vec![1, 2, 3, 4]
            .into_iter()
            .group_by_linq(|n| n % 2, |n| n * 100);
        assert_eq!(grouped, vec![(1, vec![100, 300]), (0, vec![200, 400])]);
    }

    #[test]
    fn set_operations() {
        let except = vec![1, 2, 3].into_iter().except(vec![2]).to_list();
        assert_eq!(except, vec![1, 3]);
        let intersect = vec![1, 2, 3].into_iter().intersect(vec![2, 3, 4]);
        assert_eq!(intersect, vec![2, 3]);
        let union = vec![1].into_iter().union_all(vec![1, 2]).to_list();
        assert_eq!(union, vec![1, 1, 2]);
        let distinct = vec![1, 1, 2].into_iter().distinct().to_list();
        assert_eq!(distinct, vec![1, 2]);
        let chunked = vec![1, 2, 3, 4].into_iter().chunk_linq(2);
        assert_eq!(chunked, vec![vec![1, 2], vec![3, 4]]);
        let count = vec![1, 2, 3, 4].into_iter().count_where(|n| n > 2);
        assert_eq!(count, 2);
        let agg = vec![1, 2, 3].into_iter().aggregate(0, |acc, n| acc + n);
        assert_eq!(agg, 6);
        let first = vec![].into_iter().first_or_default(7);
        assert_eq!(first, 7);
        let last = vec![1, 2].into_iter().last_or_default(9);
        assert_eq!(last, 2);
        let skipped = vec![1, 2, 3, 4]
            .into_iter()
            .skip_linq(1)
            .take_linq(2)
            .to_list();
        assert_eq!(skipped, vec![2, 3]);
        let with_index = vec![10, 20]
            .into_iter()
            .select_with_index(|i, v| format!("{i}:{v}"))
            .to_list();
        assert_eq!(with_index, vec!["0:10", "1:20"]);
        let where_index = vec![10, 20, 30]
            .into_iter()
            .where_with_index(|i, v| i > 0 && v > 10)
            .to_list();
        assert_eq!(where_index, vec![20, 30]);
    }

    #[test]
    fn except_returns_distinct_items() {
        // LINQ Except yields each surviving value once.
        let result = vec![1, 1, 2, 3].into_iter().except(vec![3]).to_list();
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn order_by_descending_is_stable() {
        // LINQ OrderByDescending is stable: ties keep their source order.
        let sorted = vec![("a", 1), ("b", 1), ("c", 2)]
            .into_iter()
            .order_by_descending(|(_, n)| *n);
        assert_eq!(sorted, vec![("c", 2), ("a", 1), ("b", 1)]);
    }
}
