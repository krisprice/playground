extern crate itertools;
use itertools::Itertools;
use std::cmp::{Ord, min, max};

fn merge_intervals<T: Copy + Ord>(mut intervals: Vec<(T, T)>) -> Vec<(T, T)> {
    // Sort by (end, start) because we work backwards below.
    intervals.sort_by_key(|k| (k.1, k.0)); 

    // Work backwards from the end of the list to the front.
    let mut i = intervals.len()-1;
    while i >= 1 {
        let (l_start, l_end) = intervals[i-1];
        let (r_start, r_end) = intervals[i];
        
        if r_start <= l_end {
            intervals[i-1].0 = min(l_start, r_start);
            intervals[i-1].1 = max(l_end, r_end);
            intervals.remove(i);
        }
        i -= 1;
    }
    intervals
}

fn merge_intervals_itertools(mut intervals: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
    intervals.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));

    let merged = intervals.into_iter().coalesce(|a, b|
        if a.1 >= b.0 { Ok((a.0, max(a.1, b.1))) }
        else { Err((a, b)) }
    ).collect::<Vec<(u32, u32)>>();

    merged
}

fn main() {
    let v = vec![
        (0, 1), (1, 2), (2, 3), // should merge to (0, 3)
        (11, 12), (13, 14), (10, 15), (11, 13), // should merge to (10, 15)
        (20, 25), (24, 29) // should merge to (20, 29)
    ];

    println!("Before: {:?}", v);

    let intervals = merge_intervals(v.clone());
    println!("After: {:?}", intervals);

    let intervals = merge_intervals_itertools(v.clone());
    println!("After: {:?}", intervals);

    let vv = vec![
        ([0, 1], [0, 2]), ([0, 2], [0, 3]), ([0, 0], [0, 1]),
        ([10, 15], [11, 0]), ([10, 0], [10, 16])
    ];

    println!("Before: {:?}", vv);
    let intervals = merge_intervals(vv.clone());
    println!("After: {:?}", intervals);
}
