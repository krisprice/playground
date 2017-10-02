// Created external crate and moved all types and methods there.
extern crate ipnet;
use ipnet::IpNet;

fn main() {
    let strings = vec![
        "10.0.0.0/24", "10.0.1.0/24", "10.0.1.1/24", "10.0.1.2/24",
        "10.0.2.0/24",
        "10.1.0.0/24", "10.1.1.0/24",
        "192.168.0.0/24", "192.168.1.0/24", "192.168.2.0/24", "192.168.3.0/24",
        "fd00::/32", "fd00:1::/32",
    ];

    let ipnets: Vec<IpNet> = strings.iter().filter_map(|p| p.parse().ok()).collect();
    
    println!("\nInput IP prefix list:");
    
    for n in &ipnets {
        println!("{}", n);
    }
    
    println!("\nAggregated IP prefixes:");
    
    for n in IpNet::aggregate(&ipnets) {
        println!("{}", n);
    }
}
