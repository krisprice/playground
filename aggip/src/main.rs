// Allow these while hacking.
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::cmp::{min, max};
use std::net::{Ipv4Addr};

// TODO: Implement a proper test suite.
//
// TODO: Try an implementation that uses a Trie. The idea is we would
// load the prefixes into the Trie, and when a node has full children
// we delete the children and turn that parent node into the aggregate.
// Doing it this way seems more elegant, and saves repeatedly looping
// over a merge function.
//
// TODO: Fix Ipv4Prefix up so that its pub and precompute private vars
// for start, end, etc. Also make it generic for IPv4 and IPv6. Make the
// constructor take a single string in the format 'addr/len' instead, or
// an integer for a single /32. Add a method for setting prefix so we can
// build a prefix from an int and chain fluently to set the prefix.

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Clone)]
struct Ipv4Prefix {
    address: Ipv4Addr,
    length: u8,
}

impl Ipv4Prefix {
    fn new<S>(address: S, length: u8) -> Ipv4Prefix
        where S: Into<String>
    {
        Ipv4Prefix {
            address: address.into().parse().unwrap(), // TODO: Don't use unwrap.
            length: length,
        }
    }

    fn start(&self) -> u32 {
        u32::from(self.address) & !((0xffffffff as u32) >> self.length)
    }

    fn end(&self) -> u32 {
        u32::from(self.address) | ((0xffffffff as u32) >> self.length)
    }

    // Return a new prefix based on this one with a shorter length. This
    // is just a helper function for use in our checks later.
    fn expanded(&self) -> Ipv4Prefix {
        Ipv4Prefix { address: self.address, length: self.length-1, }
    }
}

/// Merge prefixes.
//
// TODO: This is ugly and needs to be converted to be more idomatic.
// Doing a nested iteration that requires advancing the iterator
// inside the inner loop is giving me a serious borrow checker
// headache right now. So for now we'll do it this ugly way and come
// back to that. When we do we'll make it process our vector and
// return a new vector of the merged prefixes. At the moment this is
// modifying the original vector.

fn run_method1(prefixes: &Vec<Ipv4Prefix>) -> Vec<Ipv4Prefix> {
    // Clone the input prefix list since we'll be modifying it.
    let mut prefixes = prefixes.clone();

    // Sort by the truncated address and length. We don't need to
    // include the address in the key, but it will reduce confusion
    // when people see the output and see prefixes with non-subnet
    // addresses out of order.
    prefixes.sort_by_key(|k| (k.start(), k.address, k.length));

    let mut prev_len = 0;
    while prefixes.len() != prev_len {
        prev_len = prefixes.len();
    
        let mut i = 0;
        while i < prefixes.len() {
            while i+1 < prefixes.len() {
                if prefixes[i+1].end() <= prefixes[i].end() {
                    prefixes.remove(i+1);
                }
                else if prefixes[i+1].start() <= prefixes[i].end()+1 && prefixes[i+1].end() == prefixes[i].expanded().end() {
                    prefixes[i] = prefixes[i].expanded();
                    prefixes.remove(i+1);
                }
                else { break; }
            }
            i += 1;
        }
    }

    prefixes
}

/// Method 1 done with a reverse loop (see aggip.py)

fn run_method1_rev(prefixes: &Vec<Ipv4Prefix>) -> Vec<Ipv4Prefix> {
    let mut prefixes = prefixes.clone();
    //prefixes.sort_by_key(|k| (k.start(), k.address, k.length));
    //prefixes.reverse();
    prefixes.sort_by(|a, b| b.cmp(a));

    let mut prev_len = 0;
    while prefixes.len() != prev_len {
        prev_len = prefixes.len();
    
        let mut i = prefixes.len() - 1;
        while i >= 1 {
            if prefixes[i].end() >= prefixes[i-1].end() {
                prefixes[i-1] = prefixes.remove(i);
            }
            else if prefixes[i].end()+1 >= prefixes[i-1].start() && prefixes[i].expanded().end() == prefixes[i-1].end() {
                prefixes[i-1] = prefixes.remove(i).expanded();
            }
            i -= 1;
        }
    }

    prefixes
}

// Method 2. Treat the prefixes as integer intervals. Mergse these
// regardless of valid network boundaries. Then split these merged
// intervals back into valid networks.

fn run_method2(prefixes: &Vec<Ipv4Prefix>) -> Vec<Ipv4Prefix> {
    // TODO: convert from using u32 to IpAddr types?
    let mut intervals: Vec<(u32, u32)> = prefixes.iter().map(|x| (x.start(), x.end()+1)).collect();
    intervals.sort();

    // TODO: Would reversing this loop to run from back to front be
    // better? Should avoid the use of continue, and probably be more
    // efficient as the vector doesn't need to move elements, and the
    // intervals.len() doesn't need to change on each iteration.

    let mut i = 0;
    while i < intervals.len() - 1 {
        let (p1_start, p1_end) = intervals[i];
        let (p2_start, p2_end) = intervals[i+1];

        if p1_end >= p2_start {
            intervals[i+1].0 = min(p1_start, p2_start);
            intervals[i+1].1 = max(p1_end, p2_end);
            intervals.remove(i);
            continue;
        }
        i+=1
    }

    let mut prefixes: Vec<Ipv4Prefix> = Vec::new();
    for (start, end) in intervals {
        let subnets = interval_to_subnets(start, end);
        //println!("start = {} end = {} subnets = {:?}", start, end, subnets);
        prefixes.extend(subnets);
    }   
    prefixes
}

// TODO: Get rid of all hard coded integer sizes, e.g. 32 below.
fn get_first_subnet(start: u32, end: u32) -> Ipv4Prefix {
    let r = end - start;
    let n = 32 - r.leading_zeros() - 1;
    let num_bits = min(n, start.trailing_zeros());
    //println!("start = {} end = {} r = {} n = {} num_bits = {}", start, end, r, n, num_bits);
    
    // TODO: See todo at Ipv4Prefix about making a constructor so we can
    // create prefixes using something like:
    //   Ipv4Prefix::new(start, 32 - num_bits);
    
    Ipv4Prefix {
        address: Ipv4Addr::from(start),
        length: (32 - num_bits) as u8,
    }
}

fn interval_to_subnets(start: u32, end: u32) -> Vec<Ipv4Prefix> {
    let mut subnets: Vec<Ipv4Prefix> = Vec::new();
    let mut new_start = start;
    while new_start < end {
        let subnet = get_first_subnet(new_start, end);
        new_start = subnet.end()+1;
        subnets.push(subnet);
    }
    subnets
}

fn main() {
    // Unordered list of prefixes. Just assume we have read this from a
    // file or database.
    let prefixes = vec![
        Ipv4Prefix::new("10.0.2.0", 24),
        Ipv4Prefix::new("10.0.1.1", 24),
        Ipv4Prefix::new("10.1.1.0", 24),
        Ipv4Prefix::new("10.1.0.0", 24),
        Ipv4Prefix::new("10.0.1.2", 24),
        Ipv4Prefix::new("10.0.0.0", 24),
        Ipv4Prefix::new("10.0.1.0", 24),
        Ipv4Prefix::new("192.168.0.0", 24),
        Ipv4Prefix::new("192.168.1.0", 24),
        Ipv4Prefix::new("192.168.2.0", 24),
        Ipv4Prefix::new("192.168.3.0", 24),
    ];

    fn print_prefixes(prefixes: &Vec<Ipv4Prefix>) {
        for p in prefixes {
            println!("\t{}/{} u32: {} start: {} end: {}", p.address, p.length, u32::from(p.address), p.start(), p.end());
        }
    }    
    
    println!("\nBefore aggregation:\n");
    print_prefixes(&prefixes);

    let prefixes1 = run_method1(&prefixes);
    let prefixes1_rev = run_method1_rev(&prefixes);
    let prefixes2 = run_method2(&prefixes);

    println!("\nMethod 1:\n");
    print_prefixes(&prefixes1);

    println!("\nMethod 1 rev:\n");
    print_prefixes(&prefixes1_rev);

    println!("\nMethod 2:\n");
    print_prefixes(&prefixes2);
}
