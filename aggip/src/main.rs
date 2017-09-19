use std::fmt::Display;
use std::option::Option::{Some, None};
use std::str::FromStr;

// Created external crate for IpNet types and the aggregate method and
// moved everything there.
extern crate ipnet;
use ipnet::{IpNet, Ipv4Net, Ipv6Net};

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
    println!("\nCombined list:");
    print_ipnet_vec(&ipnets);
    println!("\nIPv4 only list:");
    print_ipnet_vec(&ipv4nets);
    println!("\nIPv6 only list:");
    print_ipnet_vec(&ipv6nets);

    println!("\nAfter aggregation:");
    println!("\nCombined list:");
    print_ipnet_vec(&IpNet::aggregate(&ipnets));
    println!("\nIPv4 only list:");
    print_ipnet_vec(&Ipv4Net::aggregate(&ipv4nets));
    println!("\nIPv6 only list:");
    print_ipnet_vec(&Ipv6Net::aggregate(&ipv6nets));
}
