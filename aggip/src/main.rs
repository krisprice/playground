use std::option::Option::{Some, None};
use std::cmp::{min, max};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

// Created external crate for IpNet types and moved everything there.
extern crate ipnet;
use ipnet::*;

extern crate itertools;

// Perhaps it's time to make some operations for IpAddrs so we can Add,
// Sub, etc. on them?
pub trait IpAddrOps<RHS> {
    type Output;
    fn add(self, rhs: RHS) -> Self::Output;
    fn sub(self, rhs: RHS) -> Self::Output;
}

trait IpNetUtils {
    fn print(&self);
    fn agg(&self) -> Self;
}

impl IpNetUtils for Vec<Ipv4Net> {
    fn print(&self) {
        for n in self {
            println!("{} netmask={} hostmask={} network={} broadcast={}", n, n.netmask(), n.hostmask(), n.network(), n.broadcast());
        }
    }

    fn agg(&self) -> Vec<Ipv4Net> {
        aggregate_networks_v4(self)
    }
}

impl IpNetUtils for Vec<Ipv6Net> {
    fn print(&self) {
        for n in self {
            println!("{} netmask={} hostmask={} network={} broadcast={}", n, n.netmask(), n.hostmask(), n.network(), n.broadcast());
        }
    }

    fn agg(&self) -> Vec<Ipv6Net> {
        Vec::new()
        //aggregate_networks_v6(self)
    }
}

fn aggregate_networks_v4(networks: &Vec<Ipv4Net>) -> Vec<Ipv4Net> {
    // Code below assumes increment to the broadcast address.
    let mut intervals: Vec<(u32, u32)> = networks.iter().map(|n| (u32::from(n.network()), u32::from(n.broadcast())+1)).collect();

    // Sort by (end, start) because we work backwards below.
    intervals.sort_by_key(|k| (k.1, k.0));

    // Work backwards from the end of the list to the front.
    let mut i = intervals.len()-1;
    while i >= 1 {
        // It would be nice to take l_start and l_end as references here
        // instead of indexing into intervals[] to update them. But this
        // would mean we can't do intervals.remove().
        let (l_start, l_end) = intervals[i-1];
        let (r_start, r_end) = intervals[i];
        
        if r_start <= l_end {
            intervals[i-1].0 = min(l_start, r_start);
            intervals[i-1].1 = max(l_end, r_end);
            intervals.remove(i);
        }
        i -= 1;
    }
    
    let mut res: Vec<Ipv4Net> = Vec::new();
    
    // Break up merged intervals into the largest subnets that will fit.
    for (start, end) in intervals {
        let mut new_start = start;
        while new_start < end {
            let r = end - new_start;
            let n = 32u32.saturating_sub(r.leading_zeros()).saturating_sub(1);
            let prefix_len = 32 - min(n, new_start.trailing_zeros());
            res.push(Ipv4Net::new(Ipv4Addr::from(new_start), prefix_len as u8));
            new_start += 2u32.pow(32-prefix_len);
        }
    }
    res
}

fn merge_intervals(intervals: Vec<T>) -> Vec<T> {
    /*let mut i = intervals.len()-1;
    while i >= 1 {
        let (l_start, l_end) = intervals[i-1];
        let (r_start, r_end) = intervals[i];
        
        if r_start <= l_end {
            intervals[i-1].0 = min(l_start, r_start);
            intervals[i-1].1 = max(l_end, r_end);
            intervals.remove(i);
        }
        i -= 1;
    }*/

    intervals
}

/*fn aggregate_networks_v6(networks: &Vec<Ipv6Net>) -> Vec<Ipv6Net> {
    // Code below assumes increment to the broadcast address.
    let mut intervals: Vec<([u64; 2], [u64; 2])> = networks.iter().map(|n| (u32::from(n.network()), u32::from(n.broadcast())+1)).collect();

    // Sort by (end, start) because we work backwards below.
    intervals.sort_by_key(|k| (k.1, k.0));

    // Work backwards from the end of the list to the front.
    let mut i = intervals.len()-1;
    while i >= 1 {
        // It would be nice to take l_start and l_end as references here
        // instead of indexing into intervals[] to update them. But this
        // would mean we can't do intervals.remove().
        let (l_start, l_end) = intervals[i-1];
        let (r_start, r_end) = intervals[i];
        
        if r_start <= l_end {
            intervals[i-1].0 = min(l_start, r_start);
            intervals[i-1].1 = max(l_end, r_end);
            intervals.remove(i);
        }
        i -= 1;
    }
    
    let mut res: Vec<Ipv4Net> = Vec::new();
    
    // Break up merged intervals into the largest subnets that will fit.
    for (start, end) in intervals {
        let mut new_start = start;
        while new_start < end {
            let r = end - new_start;
            let n = 32u32.saturating_sub(r.leading_zeros()).saturating_sub(1);
            let prefix_len = 32 - min(n, new_start.trailing_zeros());
            res.push(Ipv4Net::new(Ipv4Addr::from(new_start), prefix_len as u8));
            new_start += 2u32.pow(32-prefix_len);
        }
    }
    res
}*/

fn main() {
    let strings = vec![
        "10.0.0.0/24", "10.0.1.0/24", "10.0.1.1/24", "10.0.1.2/24",
        "10.0.2.0/24",
        "10.1.0.0/24", "10.1.1.0/24",
        "192.168.0.0/24", "192.168.1.0/24", "192.168.2.0/24", "192.168.3.0/24",
        "fd00::/32", "fd00:1::/32",
    ];

    let nets: Vec<IpNet> = strings.iter().map(|p| IpNet::from_str(p).unwrap()).collect();

    let ipv4nets: Vec<Ipv4Net> = nets.iter().filter_map(|p| if let IpNet::V4(x) = *p { Some(x) } else { None }).collect();
    let ipv6nets: Vec<Ipv6Net> = nets.iter().filter_map(|p| if let IpNet::V6(x) = *p { Some(x) } else { None }).collect();

    println!("Before aggregation:");

    ipv4nets.print();
    ipv6nets.print();
    let aggs = ipv4nets.agg();

    println!("After aggregation:");
    aggs.print();
}
