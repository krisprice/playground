`coalesce_intervals()` followed by `merge_intervals_forwards_copy()` and `merge_intervals_backwards_copy()` perform the best. `coalesce_intervals()` and `merge_intervals_forwards_copy()` are essentially the same thing.

```
kprice@KPRICE-X1:/mnt/c/code/playground/merge_intervals$ cargo +nightly bench _u32; cargo +nightly bench _ipv6addr
    Finished release [optimized] target(s) in 0.0 secs
     Running target/release/deps/merge_intervals-2390a4b9a8215bff

running 6 tests
test tests::bench_coalesce_intervals_u32                    ... bench:       1,266 ns/iter (+/- 101)
test tests::bench_merge_intervals_backwards_copy_u32        ... bench:       1,392 ns/iter (+/- 130)
test tests::bench_merge_intervals_backwards_remove_u32      ... bench:       2,906 ns/iter (+/- 361)
test tests::bench_merge_intervals_forwards_remove_u32       ... bench:      32,285 ns/iter (+/- 2,655)
test tests::bench_merge_intervals_fowards_copy_u32          ... bench:       1,275 ns/iter (+/- 104)
test tests::bench_merge_intervals_itertools_u32             ... bench:       1,983 ns/iter (+/- 169)

test result: ok. 0 passed; 0 failed; 0 ignored; 6 measured; 7 filtered out

    Finished release [optimized] target(s) in 0.0 secs
     Running target/release/deps/merge_intervals-2390a4b9a8215bff

running 6 tests
test tests::bench_coalesce_intervals_ipv6addr               ... bench:      34,030 ns/iter (+/- 4,608)
test tests::bench_merge_intervals_backwards_copy_ipv6addr   ... bench:      32,782 ns/iter (+/- 3,115)
test tests::bench_merge_intervals_backwards_remove_ipv6addr ... bench:      34,859 ns/iter (+/- 3,111)
test tests::bench_merge_intervals_forwards_remove_ipv6addr  ... bench:     147,991 ns/iter (+/- 10,446)
test tests::bench_merge_intervals_fowards_copy_ipv6addr     ... bench:      33,204 ns/iter (+/- 3,455)
test tests::bench_merge_intervals_itertools_ipv6addr        ... bench:      37,466 ns/iter (+/- 3,194)

test result: ok. 0 passed; 0 failed; 0 ignored; 6 measured; 7 filtered out
```
