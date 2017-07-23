// Allow these while hacking.
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::net::{Ipv4Addr};

// TODO: Implement a proper test suite.
//
// TODO: Try an implementation that uses a Trie. The idea is we would
// load the prefixes into the Trie, and when a node has full children
// we delete the children and turn that parent node into the aggregate.
// Doing it this way seems more elegant, and saves repeatedly looping
// over a merge function.

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord)]
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

fn merge_prefixes(prefixes: &mut Vec<Ipv4Prefix>) {  
    let mut i = 0;
    while i < prefixes.len() {
        println!("{:?}", prefixes[i]);
        
        while i+1 < prefixes.len() {
            println!("\t{:?}", prefixes[i+1]);

            // We know that q.start() is always greater than p.start()
            // because we sorted our prefixes earlier.
            if prefixes[i+1].end() <= prefixes[i].end() {
                println!("\tp contains q, removing q");
                prefixes.remove(i+1);
            }
            // TODO: Not sure this is bug free. Need to think it through and write some proper tests.
            else if prefixes[i+1].start() <= prefixes[i].end()+1 && prefixes[i+1].end() == prefixes[i].expanded().end() {
                println!("\tp.expanded() contains q, removing q");
                prefixes[i] = prefixes[i].expanded();
                prefixes.remove(i+1);
            }
            else { break; }
        }
        i += 1;
    }
}

fn main() {
    // Unordered list of prefixes. Just assume we have read this from a
    // file or database.
    let mut prefixes = vec![
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

    // Sort by the truncated address and length. We don't need to
    // include the address in the key, but it will reduce confusion
    // when people see the output and see prefixes with non-subnet
    // addresses out of order.
    prefixes.sort_by_key(|k| (k.start(), k.address, k.length));

    // Print before.
    println!("\nHere's our sorted data before aggregation:\n");
    for p in &prefixes {
        println!("\t{}/{} u32: {} start: {} end: {}", p.address, p.length, u32::from(p.address), p.start(), p.end());
    }
    println!();

    // Merge.
    // TODO: Ick, do we need to waste a whole iteration just to find out we are done? 
    let mut prev_len = 0;
    while prefixes.len() != prev_len {
        prev_len = prefixes.len();
        merge_prefixes(&mut prefixes);
    }

    // Print after.
    println!("\nHere's our data after aggregation:\n");
    for p in &prefixes {
        println!("\t{}/{} {} start: {} end: {}", p.address, p.length, u32::from(p.address), p.start(), p.end());
    }
    println!();
}
