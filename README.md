# linqguru

**LINQ-style query methods for Rust iterators** — 20 C#-named extension methods over any `Iterator`: `where_`, `select`, `order_by`, `group_by_linq`, `aggregate`, `chunk_linq` and more. Zero dependencies.

For C# developers who miss LINQ vocabulary when writing Rust, and for anyone who prefers a fluent query pipeline over nested `filter().map().collect()` incantations.

## Usage

```rust
use linqguru::LinqExt;

fn main() {
    let result = vec![1, 2, 3, 4, 5]
        .into_iter()
        .where_(|n| n % 2 == 0)
        .select(|n| n * 10)
        .to_list();
    assert_eq!(result, vec![20, 40]);

    let grouped = vec![1, 2, 3, 4]
        .into_iter()
        .group_by_linq(|n| n % 2, |n| n.to_string());
    println!("{grouped:?}");
}
```

## Methods

`where_` · `where_with_index` · `select` · `select_with_index` · `order_by` · `order_by_descending` · `group_by_linq` (preserves encounter order) · `distinct` · `first_or_default` · `last_or_default` · `count_where` · `aggregate` · `to_list` · `except` · `intersect` · `union_all` · `union` · `skip_linq` · `take_linq` · `chunk_linq`

Standard `Iterator` methods (`map`, `filter`, `sum`, `any`, `all`, `zip`, `take`, `skip`) remain available — `LinqExt` only adds the LINQ vocabulary on top.

## License

Licensed under either of MIT or Apache-2.0, at your option.

Name is a tribute to LINQ (Language Integrated Query, .NET); this crate is an independent implementation for Rust iterators.
