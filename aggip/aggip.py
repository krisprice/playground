"""Prototyping methods for aggregating IP prefixes, which is basically
aggregating integer intervals with some special bounds. I'm using the
standard Python ipaddress library that gives us IP address types with
support for comparisons, conversions, etc. Such types usually exist in
some useful form in most languages. Just prototyping with IPv4 only."""

from ipaddress import *
import cProfile

def run_method1(prefixes, reduce_func):
    """Sort the list in order from lowest to highest network address and
    shortest to longest prefix length. Run a reduction function. Repeat
    this until the list stops getting smaller. This does need to waste a
    whole iteration just to verify we can't reduce the list anymore."""

    # Take a copy, so we don't modify the original.
    prefixes = sorted(prefixes)
    
    prev_len = 0
    while len(prefixes) != prev_len:
        prev_len = len(prefixes)
        prefixes = reduce_func(prefixes)
    
    return prefixes

def reduce_prefixes1(prefixes):
    """For each prefix in the list, scan forward merging as many
    subsequent prefixes as possible. Delete the prefixes that are
    merged."""

    i = 0
    while i < len(prefixes): # Should replace this with an i, p = enumerate(prefixes)?
        p = prefixes[i]      # See next method, would it also be better to work backwards
                                # in this loop?

        while i+1 < len(prefixes):
            next_p = prefixes[i+1]

            if p.broadcast_address >= next_p.broadcast_address:
                del prefixes[i+1]
            elif p.broadcast_address+1 >= next_p.network_address and p.supernet().broadcast_address == next_p.broadcast_address:
                prefixes[i] = p = p.supernet()
                del prefixes[i+1]
            else:
                break
        i+=1
    return prefixes

def reduce_prefixes1_reverse_loop(prefixes):
    """Go in the reverse direction. If the first prefix is very large we
    save some work by not moving all the items in the list while merging
    this prefix. This won't be very significant."""

    prefixes.reverse()
    
    i = len(prefixes)-1
    while i >= 1:
        p = prefixes[i]
        next_p = prefixes[i-1]

        if p.broadcast_address >= next_p.broadcast_address:
            prefixes[i-1] = prefixes[i] # If we change the sort to largest to smallest address and shortest to longest prefix
                                        # we could do away with this copy couldn't we?
            del prefixes[i]
        elif p.broadcast_address+1 >= next_p.network_address and p.supernet().broadcast_address == next_p.broadcast_address:
            prefixes[i-1] = p.supernet()
            del prefixes[i]
        i-=1
    
    prefixes.reverse()
    return prefixes

def reduce_prefixes1_consume_list(prefixes):
    """Expects input prefixes to be a sorted list. Consumes the input.
    Returns a new sorted list as the result."""
    
    merged = []
    prefixes.reverse()
    p = prefixes.pop()
    pp = p.supernet()

    while prefixes:
        next_p = prefixes.pop()
        if p.broadcast_address >= next_p.broadcast_address:
            continue
        elif p.broadcast_address+1 >= next_p.network_address and pp.broadcast_address == next_p.broadcast_address:
            p = pp
        else:
            merged.append(p)
            p = next_p
            pp = p.supernet()
    
    merged.append(p)
    return merged

from collections import deque

def run_method1_deque(prefixes):
    """See reduce_prefixes1_deque()"""
    prefixes = deque(sorted(prefixes))
    
    prev_len = 0
    while len(prefixes) != prev_len:
        prev_len = len(prefixes)
        prefixes = reduce_prefixes1_deque(prefixes)
    
    return prefixes

def reduce_prefixes1_deque(prefixes):
    """In the first two we are modifying the list by deleting items in
    the middle. This presumably forces a lot of memcpy work inside the
    list. Using a linked list should be faster. Python doesn't have a
    native linked list so we're using a deque which is backed by one.
    Deleting items in the deque using indexes doesn't work well because
    deque seems to scan through the list. Working with regular style of
    linked list using pointers is ideally what we want. But in lieu of
    that we consume the input, and create a new deque of the output.
    This effectively becomes reduce_prefixes1_consume_list() above."""

    merged = deque()
    p = prefixes.popleft()
    pp = p.supernet()
    
    while prefixes:
        next_p = prefixes.popleft()
        if p.broadcast_address >= next_p.broadcast_address:
            continue
        elif p.broadcast_address+1 >= next_p.network_address and pp.broadcast_address == next_p.broadcast_address:
            p = pp
        else:
            merged.append(p)
            p = next_p
            pp = p.supernet()
    
    merged.append(p)
    return merged

from heapq import *

def run_method1_heapq(prefixes):
    """See reduce_prefixes1_heap()"""
    prefixes = prefixes.heapify(list(prefixes))
    
    prev_len = 0
    while len(prefixes) != prev_len:
        prev_len = len(prefixes)
        prefixes = reduce_prefixes1_heapq(prefixes)
    
    return prefixes

def trailing_zeros(n):
    b = bin(n)
    r = len(b) - len(b.rstrip('0'))
    return r

def get_first_subnet(start, end):
    l = 32 if start.version == 4 else 128
    r = int(end) - int(start)
    n = r.bit_length()-1
    num_bits = l - min(n, trailing_zeros(int(start)))
    return ip_network(start).supernet(new_prefix=num_bits)

def coalesce_intervals(intervals):
    """Reduce a list of intervals by merging them if they overlap.
    Expects the input to be sorted. Output as a generator."""
    
    start, end = intervals[0]
    for next_start, next_end in intervals:
        if end >= next_start:
            start, end = (min(start, next_start), max(end, next_end))
        else:
            yield start, end
            start = next_start
            end = next_end
    yield start, end

def ipaddress_interval_to_subnets(start, end):
    """Expects inputs to be ipaddress objects. Output as a generator."""
    new_start = start
    while (new_start < end):
        subnet = get_first_subnet(new_start, end)
        new_start = subnet.broadcast_address+1
        yield subnet

def run_method2(prefixes):
    """We will treat the prefixes as integer intervals. Merge these
    intervals regardless of valid network boundaries. Then in split
    these merged intervals up into valid networks."""

    intervals = coalesce_intervals([(p.network_address, p.broadcast_address+1) for p in sorted(prefixes)])
    return [n for l in map(lambda i: ipaddress_interval_to_subnets(*i), intervals) for n in l]

if __name__ == "__main__":
    prefixes = [
        ip_network("10.0.2.0/24"),
        ip_network("10.0.1.1/24", False),
        ip_network("10.1.1.0/24"),
        ip_network("10.1.0.0/24"),
        ip_network("10.0.1.2/24", False),
        ip_network("10.0.0.0/24"),
        ip_network("10.0.1.0/24"),
        ip_network("192.168.0.0/24"),
        ip_network("192.168.1.0/24"),
        ip_network("192.168.2.0/24"),
        ip_network("192.168.3.0/24"),
    ]
    
    def print_prefixes(prefixes):
        for p in sorted(prefixes):
            print("\t{}".format(p))

    print("Before merging:")
    print_prefixes(prefixes)

    print("reduce_prefixes1():")
    print_prefixes(run_method1(prefixes, reduce_prefixes1))
    
    print("reduce_prefixes1_reverse_loop():")
    print_prefixes(run_method1(prefixes, reduce_prefixes1_reverse_loop))

    print("reduce_prefixes1_consume_list():")
    print_prefixes(run_method1(prefixes, reduce_prefixes1_consume_list))

    print("reduce_prefixes1_deque():")
    print_prefixes(run_method1_deque(prefixes))

    print("run_method2():")
    print_prefixes(run_method2(prefixes))


    # Add lots of prefixes for profiling
    prefixes.extend(ip_network('10.0.0.0/20').subnets(prefixlen_diff=10))
    prefixes.extend(ip_network('10.1.0.0/20').subnets(prefixlen_diff=10))
    prefixes.extend(ip_network('10.2.0.0/20').subnets(prefixlen_diff=10))
    prefixes.extend(ip_network('10.3.0.0/20').subnets(prefixlen_diff=10))
    prefixes.extend(ip_network('10.4.0.0/20').subnets(prefixlen_diff=10))
    prefixes.extend(ip_network('10.5.0.0/20').subnets(prefixlen_diff=10))
    prefixes.extend(ip_network('10.6.0.0/20').subnets(prefixlen_diff=10))
    prefixes.extend(ip_network('10.7.0.0/20').subnets(prefixlen_diff=10))

    # Wow, method2 is about 4x faster than method1. Reverse loop doesn't
    # make a big difference (somewhat expected). Linked list is better,
    # but still not as good as method2. Consume list is the best version
    # of method1.
    #cProfile.run('run_method1(prefixes, reduce_prefixes1_consume_list)', sort='tottime')
    #cProfile.run('run_method2(prefixes)', sort='tottime')
    #cProfile.run('run_method1(prefixes, reduce_prefixes1)', sort='tottime')   
    #cProfile.run('run_method1(prefixes, reduce_prefixes1_reverse_loop)', sort='tottime')
    #cProfile.run('run_method1_deque(prefixes)', sort='tottime')
    