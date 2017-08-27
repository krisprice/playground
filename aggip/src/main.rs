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
    print_ipnet_vec(&IpNet::aggregate(&ipnets));
    print_ipnet_vec(&Ipv4Net::aggregate(&ipv4nets));
    print_ipnet_vec(&Ipv6Net::aggregate(&ipv6nets));

    // Test subnets iterator
    /*let v: Vec<Ipv4Net> = Ipv4Net::from_str("10.1.1.0/24").unwrap().new_subnets(26).into_iter().collect();
    println!("{:?}", v);

    let i = Ipv4Net::from_str("10.1.1.0/24").unwrap().new_subnets(8);
    for ip in i {
        println!("{}", ip);
    }
    let i = Ipv6Net::from_str("fd00::/16").unwrap().new_subnets(18);
    for ip in i {
        println!("{}", ip);
    }
    let h = Ipv4Net::from_str("10.1.1.0/24").unwrap().hosts();
    for ip in h {
        println!("{}", ip);
    }

    let h = Ipv6Net::from_str("fd00::/125").unwrap().hosts();
    for ip in h {
        println!("{}", ip);
    }*/


    // TODO:
    // * impl Range for Emu128 as a test
    // * Create a custom IpRange trait for requires IpAdd trait
    // * Can the Range and step_by make sense for subnets()?
    // would be good to have Add and Step impl for IpAddr so can use Range
    // Range and Step are not stable so stick with custom iterator until
    // they are

    use std::ops::Range;

    let r = 1..5;
    println!("{} {}", r.start, r.end);

    /*let mut r = Range {
        start: Ipv4Addr::from_str("10.1.1.1").unwrap(),
        end: Ipv4Addr::from_str("10.1.1.20").unwrap(),
    };

    for i in r {
        println!("{}", i);
    }*/
    
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct NewType(u32);

    // I get the 
    impl std::ops::Add for NewType {
        type Output = NewType;
        fn add(self, other: NewType) -> NewType {
            NewType(self.0 + other.0)
        }
    }

    // This is marked nightly-only and so generates the "error: use of
    // unstable library feature 'step_trait': likely to be replaced by
    // finer-grained traits (see issue #42168)." Yet I can use other
    // Ranges (e.g. just doing 1..5) that also must've implemented
    // this trait. How come those are allowed?
    //impl std::iter::Step for NewType {
        // ...
    //}

    let a = NewType(10);
    let b = NewType(20);

    for i in a..b {
        println!("{:?}", i);
    }

}
