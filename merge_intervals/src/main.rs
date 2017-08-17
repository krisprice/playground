#![allow(unused_mut)]
#![allow(unused_variables)]
extern crate itertools;

fn merge_intervals1() {
    let v = vec![
        (0, 1), (1, 2), (2, 3), // should merge to (0, 3)
        (11, 12), (13, 14), (10, 15), (11, 13), // should merge to (10, 15)
        (20, 25), (24, 29) // should merge to (20, 29)
    ];

    let mut intervals = v.clone();

    // Sort by (end, start) because we work backwards below.
    intervals.sort_by_key(|k| (k.1, k.0)); 
    
    println!("Before: {:?}", v);
    println!("Sorted: {:?}", intervals);

    // Work backwards from the end of the list to the front.
    let mut i = intervals.len()-1;
    while i >= 1 {
        // It would be nice to take l_start and l_end as references here
        // instead of indexing into intervals[] to update them. But this
        // would mean we can't do intervals.remove().
        let (l_start, l_end) = intervals[i-1];
        let (r_start, r_end) = intervals[i];
        
        if r_start <= l_end {
            intervals[i-1].0 = std::cmp::min(l_start, r_start);
            intervals[i-1].1 = std::cmp::max(l_end, r_end);
            intervals.remove(i);
        }

        i -= 1;
    }

    println!("After: {:?}", intervals);
}

fn merge_intervals2() {
    let v = vec![
        (0, 1), (1, 2), (2, 3), // should merge to (0, 3)
        (11, 12), (13, 14), (10, 15), (11, 13), // should merge to (10, 15)
        (20, 25), (24, 29) // should merge to (20, 29)
    ];

    let mut intervals = v.clone();
    intervals.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));

    println!("Before: {:?}", v);
    println!("Sorted: {:?}", intervals);

    use itertools::Itertools;

    let merged = intervals.into_iter().coalesce(|a, b|
        if a.1 >= b.0 { Ok((a.0, std::cmp::max(a.1, b.1))) }
        else { Err((a, b)) }
    ).collect::<Vec<(i64, i64)>>();

    println!("After: {:?}", merged);
}

fn main() {
    merge_intervals1();
    merge_intervals2();
}
