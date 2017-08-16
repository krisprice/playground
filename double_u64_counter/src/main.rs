fn main() {
    let mut counter = [0u64, u64::max_value() - 5];
    let finish = [1u64, 5u64];
    
    println!("Before: counter={:?}", counter);

    let mut i = 1;
    while counter < finish {
        let (lo, overflowed) = counter[1].overflowing_add(1);
        counter[1] = lo;

        if overflowed {
            counter[0] += 1;
        }

        println!("Iteration {}: hi={} lo={} overflowed={}", i, counter[0], counter[1], overflowed);
        i += 1;
    }

    println!("After: counter={:?}", counter);
}
