use std::option::Option::{Some, None};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

// Created external crate for IpNet types and moved everything there.
extern crate ipnet;
use ipnet::*;

use std::fmt::Display;
fn print_ipnet_vec<T: Display>(networks: &Vec<T>) {
    for n in networks {
        println!("{}", n);
    }
}

fn main() {
    let strings = vec![
        "10.0.0.0/24", "10.0.1.0/24", "10.0.1.1/24", "10.0.1.2/24",
        "10.0.2.0/24",
        "10.1.0.0/24", "10.1.1.0/24",
        "192.168.0.0/24", "192.168.1.0/24", "192.168.2.0/24", "192.168.3.0/24",
        "fd00::/32", "fd00:1::/32",
    ];

    let ipnets: Vec<IpNet> = strings.iter().map(|p| IpNet::from_str(p).unwrap()).collect();
    let ipv4nets: Vec<Ipv4Net> = ipnets.iter().filter_map(|p| if let IpNet::V4(x) = *p { Some(x) } else { None }).collect();
    let ipv6nets: Vec<Ipv6Net> = ipnets.iter().filter_map(|p| if let IpNet::V6(x) = *p { Some(x) } else { None }).collect();

    println!("Before aggregation:");
    print_ipnet_vec(&ipv4nets);
    print_ipnet_vec(&ipv6nets);

    println!("\nAfter aggregation:");
    print_ipnet_vec(&Ipv4Net::aggregate(&ipv4nets));
    print_ipnet_vec(&Ipv6Net::aggregate(&ipv6nets));

    let ip = ipv4nets[2];
    println!("{:?}", ip.subnets(28));
}
