#![feature(test)]
#![feature(i128_type)]
extern crate test;
use std::cmp::{Ord, min, max};
extern crate itertools;
use itertools::Itertools;

fn merge_intervals_backwards_remove<T: Copy + Ord>(mut intervals: Vec<(T, T)>) -> Vec<(T, T)> {
    // Sort by (end, start) because we work backwards below.
    intervals.sort_by_key(|k| (k.1, k.0)); 

    // Work backwards from the end of the list to the front.
    let mut i = intervals.len()-1;
    while i >= 1 {
        if intervals[i].0 <= intervals[i-1].1 {
            intervals[i-1].0 = min(intervals[i-1].0, intervals[i].0);
            intervals[i-1].1 = max(intervals[i-1].1, intervals[i].1);
            intervals.remove(i);
        }
        i -= 1;
    }
    intervals
}

fn merge_intervals_backwards_copy<T: Copy + Ord>(mut intervals: Vec<(T, T)>) -> Vec<(T, T)> {
    let mut res: Vec<(T, T)> = Vec::new();
    
    if intervals.len() == 0 {
        return res;
    }

    // Sort by (end, start) because we work backwards below.
    intervals.sort_by_key(|k| (k.1, k.0));
    let (mut start, mut end) = intervals[intervals.len()-1];

    // Work backwards from the end of the list to the front.
    let mut i = intervals.len()-1;
    while i >= 1 {
        let (next_start, next_end) = intervals[i-1];
        if start <= next_end {
            start = min(start, next_start);
            end = max(end, next_end);
        }
        else {
            res.push((start, end));
            start = next_start;
            end = next_end;
        }
        i -= 1;
    }
    res.push((start, end));
    res
}

fn merge_intervals_forwards_remove<T: Copy + Ord>(mut intervals: Vec<(T, T)>) -> Vec<(T, T)> {
    intervals.sort(); 

    let mut i = 0;
    while i < intervals.len()-1 { // It is necessary to evaluate len() each iteration because it is getting shorter.
        if intervals[i].1 >= intervals[i+1].0 {
            intervals[i+1].0 = min(intervals[i+1].0, intervals[i].0);
            intervals[i+1].1 = max(intervals[i+1].1, intervals[i].1);
            intervals.remove(i);
            continue; // Avoid incrementing i because we remove(i).
        }
        i += 1;
    }
    intervals
}

fn merge_intervals_forwards_copy<T: Copy + Ord>(mut intervals: Vec<(T, T)>) -> Vec<(T, T)> {
    let mut res: Vec<(T, T)> = Vec::new();
    
    if intervals.len() == 0 {
        return res;
    }

    intervals.sort();
    let (mut start, mut end) = intervals[0];
    
    let mut i = 1;
    let len = intervals.len();
    while i < len {
        let (next_start, next_end) = intervals[i];
        if end >= next_start {
            start = min(start, next_start);
            end = max(end, next_end);
        }
        else {
            res.push((start, end));
            start = next_start;
            end = next_end;
        }
        i += 1;
    }
    res.push((start, end));
    res
}

// As I did it in aggip.py for the generator style, but generators
// aren't stable in Rust yet. This turned out to be the fastest.
fn coalesce_intervals<T: Copy + Ord>(mut intervals: Vec<(T, T)>) -> Vec<(T, T)> {
    let mut res: Vec<(T, T)> = Vec::new();
    
    if intervals.len() == 0 {
        return res;
    }

    intervals.sort(); 
    let (mut start, mut end) = intervals[0];
    
    for (next_start, next_end) in intervals {
        if end >= next_start {
            start = min(start, next_start);
            end = max(end, next_end);
        }
        else {
            res.push((start, end));
            start = next_start;
            end = next_end;
        }
    }
    res.push((start, end));
    res
}

fn merge_intervals_itertools<T: Copy + Ord>(mut intervals: Vec<(T, T)>) -> Vec<(T, T)> {
    intervals.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));

    let merged = intervals.into_iter().coalesce(|a, b|
        if a.1 >= b.0 { Ok((a.0, max(a.1, b.1))) }
        else { Err((a, b)) }
    ).collect::<Vec<(T, T)>>();

    merged
}

#[cfg(test)]
mod tests {
    use std::convert::From;
    use std::net::{Ipv4Addr, Ipv6Addr};
    use test::Bencher;
    use super::*;
    
    #[test]
    fn main_test() {       
        let v = vec![
            (0, 1), (1, 2), (2, 3),
            (11, 12), (13, 14), (10, 15), (11, 13),
            (20, 25), (24, 29),
        ];

        let v_ok = vec![
            (0, 3),
            (10, 15),
            (20, 29),
        ];

        let mut v_ok_rev = v_ok.clone();
        v_ok_rev.reverse();

        let vv = vec![
            ([0, 1], [0, 2]), ([0, 2], [0, 3]), ([0, 0], [0, 1]),
            ([10, 15], [11, 0]), ([10, 0], [10, 16]),
        ];

        let vv_ok = vec![
            ([0, 0], [0, 3]),
            ([10, 0], [11, 0]),
        ];

        let mut vv_ok_rev = vv_ok.clone();
        vv_ok_rev.reverse();

        assert_eq!(merge_intervals_backwards_copy(v.clone()), v_ok_rev);
        assert_eq!(merge_intervals_backwards_remove(v.clone()), v_ok);
        assert_eq!(merge_intervals_forwards_copy(v.clone()), v_ok);
        assert_eq!(merge_intervals_forwards_remove(v.clone()), v_ok);
        assert_eq!(coalesce_intervals(v.clone()), v_ok);
        assert_eq!(merge_intervals_itertools(v.clone()), v_ok);

        assert_eq!(merge_intervals_backwards_copy(vv.clone()), vv_ok_rev);
        assert_eq!(merge_intervals_backwards_remove(vv.clone()), vv_ok);
        assert_eq!(merge_intervals_forwards_copy(vv.clone()), vv_ok);
        assert_eq!(merge_intervals_forwards_remove(vv.clone()), vv_ok);
        assert_eq!(coalesce_intervals(vv.clone()), vv_ok);
        assert_eq!(merge_intervals_itertools(vv.clone()), vv_ok);

        let ip = vec![
            (Ipv4Addr::from(0), Ipv4Addr::from(1)),
            (Ipv4Addr::from(1), Ipv4Addr::from(2)),
            (Ipv4Addr::from(2), Ipv4Addr::from(3)),
        ];

        let ip_ok = vec![
            (Ipv4Addr::from(0), Ipv4Addr::from(3)),
        ];

        let mut ip_ok_rev = ip_ok.clone();
        ip_ok_rev.reverse();

        let ip6 = vec![
            (Ipv6Addr::from(0), Ipv6Addr::from(1)),
            (Ipv6Addr::from(1), Ipv6Addr::from(2)),
            (Ipv6Addr::from(2), Ipv6Addr::from(3)),
        ];

        let ip6_ok = vec![
            (Ipv6Addr::from(0), Ipv6Addr::from(3)),
        ];

        let mut ip6_ok_rev = ip6_ok.clone();
        ip6_ok_rev.reverse();
        
        assert_eq!(merge_intervals_backwards_copy(ip.clone()), ip_ok_rev);
        assert_eq!(merge_intervals_backwards_remove(ip.clone()), ip_ok);
        assert_eq!(merge_intervals_forwards_copy(ip.clone()), ip_ok);
        assert_eq!(merge_intervals_forwards_remove(ip.clone()), ip_ok);
        assert_eq!(coalesce_intervals(ip.clone()), ip_ok);
        assert_eq!(merge_intervals_itertools(ip.clone()), ip_ok);

        assert_eq!(merge_intervals_backwards_copy(ip6.clone()), ip6_ok_rev);
        assert_eq!(merge_intervals_backwards_remove(ip6.clone()), ip6_ok);
        assert_eq!(merge_intervals_forwards_copy(ip6.clone()), ip6_ok);
        assert_eq!(merge_intervals_forwards_remove(ip6.clone()), ip6_ok);
        assert_eq!(coalesce_intervals(ip6.clone()), ip6_ok);
        assert_eq!(merge_intervals_itertools(ip6.clone()), ip6_ok);
    }

    macro_rules! bench_merge_func {
        ($name:ident, $f:ident) => (
            #[bench]
            fn $name(b: &mut Bencher) {
                let mut v: Vec<(u32, u32)> = Vec::new();
                let mut last = 0;
                for x in 0..25 {
                    for y in 0..25 {
                        let i = x << 16 & y;
                        v.push((last, i));
                        last = i;
                    }
                }
                b.iter(|| $f(v.clone()));
            }
        )
    }

    bench_merge_func!(bench_merge_intervals_backwards_remove_u32, merge_intervals_backwards_remove);
    bench_merge_func!(bench_merge_intervals_forwards_remove_u32, merge_intervals_forwards_remove);
    bench_merge_func!(bench_merge_intervals_backwards_copy_u32, merge_intervals_backwards_copy);
    bench_merge_func!(bench_merge_intervals_fowards_copy_u32, merge_intervals_forwards_copy);
    bench_merge_func!(bench_coalesce_intervals_u32, coalesce_intervals);
    bench_merge_func!(bench_merge_intervals_itertools_u32, merge_intervals_itertools);

    macro_rules! bench_merge_func_ipv4addr {
        ($name:ident, $f:ident) => (
            #[bench]
            fn $name(b: &mut Bencher) {
                let mut v: Vec<(Ipv4Addr, Ipv4Addr)> = Vec::new();
                let mut last = Ipv4Addr::new(0, 0, 0, 0);
                for x in 0u32..25u32 {
                    for y in 0u32..25u32 {
                        let ip = Ipv4Addr::from(x << 16 & y);
                        v.push((last, ip));
                        last = ip;
                    }
                }
                b.iter(|| $f(v.clone()));
            }
        )
    }

    bench_merge_func_ipv4addr!(bench_merge_intervals_backwards_remove_ipv4addr, merge_intervals_backwards_remove);
    bench_merge_func_ipv4addr!(bench_merge_intervals_forwards_remove_ipv4addr, merge_intervals_forwards_remove);
    bench_merge_func_ipv4addr!(bench_merge_intervals_backwards_copy_ipv4addr, merge_intervals_backwards_copy);
    bench_merge_func_ipv4addr!(bench_merge_intervals_fowards_copy_ipv4addr, merge_intervals_forwards_copy);
    bench_merge_func_ipv4addr!(bench_coalesce_intervals_ipv4addr, coalesce_intervals);
    bench_merge_func_ipv4addr!(bench_merge_intervals_itertools_ipv4addr, merge_intervals_itertools);

    macro_rules! bench_merge_func_ipv6addr {
        ($name:ident, $f:ident) => (
            #[bench]
            fn $name(b: &mut Bencher) {
                let mut v: Vec<(Ipv6Addr, Ipv6Addr)> = Vec::new();
                let mut last = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0);
                for x in 0u128..25u128 {
                    for y in 0u128..25u128 {
                        let ip = Ipv6Addr::from(x << 64 & y);
                        v.push((last, ip));
                        last = ip;
                    }
                }
                b.iter(|| $f(v.clone()));
            }
        )
    }

    bench_merge_func_ipv6addr!(bench_merge_intervals_backwards_remove_ipv6addr, merge_intervals_backwards_remove);
    bench_merge_func_ipv6addr!(bench_merge_intervals_forwards_remove_ipv6addr, merge_intervals_forwards_remove);
    bench_merge_func_ipv6addr!(bench_merge_intervals_backwards_copy_ipv6addr, merge_intervals_backwards_copy);
    bench_merge_func_ipv6addr!(bench_merge_intervals_fowards_copy_ipv6addr, merge_intervals_forwards_copy);
    bench_merge_func_ipv6addr!(bench_coalesce_intervals_ipv6addr, coalesce_intervals);
    bench_merge_func_ipv6addr!(bench_merge_intervals_itertools_ipv6addr, merge_intervals_itertools);
}
