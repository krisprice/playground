"""Prototyping methods for aggregating IP prefixes, which is basically
aggregating integer intervals with some special bounds. I'm using the
standard Python ipaddress library that gives us IP address types with
support comparisons, conversion, etc. Such types usually exist in some
useful form in most languages. Just prototyping with IPv4 only."""

from ipaddress import *
import cProfile

def run_method1(prefixes):
    """Sort the list in order from lowest to highest network address and
    shortest to longest prefix length. For each prefix in the list, scan
    forward merging as many subsequent prefixes as possible. Delete the
    prefixes that are merged. Repeat this process until the list stops
    getting smaller. This does need to waste a whole iteration just to
    verify we can't merge anymore prefixes."""

    # Take a copy, so we don't modify the original.
    prefixes = sorted(prefixes)

    prev_len = 0
    while len(prefixes) != prev_len:
        prev_len = len(prefixes)
        
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

def run_method1_rev(prefixes):
    """Try going the reverse direction through the list so we aren't
    deleting elements from the begining, which would seem to be more
    compute intensive."""

    # Take a copy, so we don't modify the original.
    prefixes = sorted(prefixes, reverse=True)

    prev_len = 0
    while len(prefixes) != prev_len:
        prev_len = len(prefixes)
        
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
    return prefixes


def trailing_zeros(n):
    b = bin(n)
    r = len(b) - len(b.rstrip('0'))
    return r

def get_first_subnet(start, end):
    r = int(end) - int(start)
    n = r.bit_length()-1
    num_bits = min(n, trailing_zeros(int(start)))
    return ip_network(start).supernet(new_prefix=32-num_bits)

def ip_address_interval_to_subnets(start, end):
    subnets = []
    new_start = start
    while (new_start < end):
        subnet = get_first_subnet(new_start, end)
        new_start = subnet.broadcast_address+1
        subnets.append(subnet)
    return subnets

def run_method2(prefixes):
    """We will treat the prefixes as integer intervals. Merge these
    intervals regardless of valid network boundaries. Then in split
    these merged intervals up into valid networks."""
    
    # Take a copy, so we don't modify the original. Note the increment
    # of the broadcast address is important as we treat an interval as
    # 'a <= x < b'. Would it would be better to treat an interval as
    # 'a <= x <= b' everywhere instead?
    intervals = [(p.network_address, p.broadcast_address+1) for p in sorted(prefixes)]

    i = 0
    while i < len(intervals)-1:
        (p1_start, p1_end) = intervals[i]
        (p2_start, p2_end) = intervals[i+1]

        if p1_end+1 >= p2_start:
            intervals[i+1] = (min(p1_start, p2_start), max(p1_end, p2_end))
            del intervals[i] # Would reversing this loop and deleting from end rather than front be nicer?
            continue         # It would avoid having to use this continue also (I think?).
                             # And len(intervals) in the while condition wouldn't change every loop.
        i+=1
    
    final_intervals = []
    for (start, end) in intervals:
        subnets = ip_address_interval_to_subnets(start, end)
        final_intervals.extend(subnets)
    return final_intervals

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
    """
    prefixes1 = run_method1(prefixes)
    prefixes1_rev = run_method1_rev(prefixes)
    prefixes2 = run_method2(prefixes)

    def print_prefixes(prefixes):
        for p in sorted(prefixes):
            print("\t{}".format(p))

    print("Before merging:")
    print_prefixes(prefixes)

    print("After merge method 1:")
    print_prefixes(prefixes1)
    print("After merge method 1_rev:")
    print_prefixes(prefixes1_rev)
    print("After merge method 2:")
    print_prefixes(prefixes2)"""

    # Add lots of prefixes for profiling
    prefixes.extend(ip_network('10.0.0.0/20').subnets(prefixlen_diff=10))
    prefixes.extend(ip_network('10.1.0.0/20').subnets(prefixlen_diff=10))
    prefixes.extend(ip_network('10.2.0.0/20').subnets(prefixlen_diff=10))
    prefixes.extend(ip_network('10.3.0.0/20').subnets(prefixlen_diff=10))
    prefixes.extend(ip_network('10.4.0.0/20').subnets(prefixlen_diff=10))
    prefixes.extend(ip_network('10.5.0.0/20').subnets(prefixlen_diff=10))
    prefixes.extend(ip_network('10.6.0.0/20').subnets(prefixlen_diff=10))
    prefixes.extend(ip_network('10.7.0.0/20').subnets(prefixlen_diff=10))

    # Wow, method2 is about 4x faster than method1.
    cProfile.run('run_method1(prefixes)')    
    cProfile.run('run_method2(prefixes)')
    # Reverse loop doesn't make a big difference.
    cProfile.run('run_method1_rev(prefixes)')
