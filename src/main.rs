// Allow these while hacking.
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use std::path::Path;

use std::net::Ipv4Addr;

extern crate byteorder;
use byteorder::{BigEndian, ReadBytesExt};

#[macro_use]
extern crate nom;
use nom::{be_u8, be_u16, be_u32};
use nom::IResult;
use nom::Needed;

/* TODO: Do I want to do it this way? Enums would be less memory
 * efficient.
 */
/*#[derive(Debug)]
enum BgpMessage {
    Open(BgpOpenMessage),
    Notification(BgpNotificationMessage),
}

named!(parse_bgp_message<BgpOpenMessage>,
    do_parse!(
        bgp_header: parse_bgp_header >>
        bgp_open_message: parse_bgp_open >>
        (bgp_open_message)
    )
);*/


// Parse BGP message header. This is common to all BGP messages.

#[derive(Debug,PartialEq)]
struct BgpMessageHeader {
    length: u16,
    message_type: u8,
}

named!(parse_bgp_header<&[u8], BgpMessageHeader>,
    do_parse!(
        tag!([0xff; 16]) >> // Marker, must be all ones. 
        
        // Length of message including this header, must be >= 19 and <= 4096.
        length: verify!(be_u16, |v: u16| v >= 19 && v <= 4096) >>
        
        // Must be 1 OPEN, 2 UPDATE, 3 NOTIFICATION, 4 KEEPALIVE, 5 ROUTE-REFRESH
        message_type: verify!(be_u8, |v: u8| v >= 1 && v <= 5) >>
        
        (BgpMessageHeader { length: length, message_type: message_type })
    )
);

// Parse BGP Open message.

#[derive(Debug,PartialEq)]
struct BgpOpenMessage {
    version: u8,
    my_autonomous_system: u16,
    hold_time: u16,
    bgp_identifier: u32,
    optional_parameters_length: u8,
    // TODO: Optional parameters not implemented
}

// TODO: Implement validation.
named!(parse_bgp_open<&[u8], BgpOpenMessage>,
    do_parse!(
        version: be_u8 >>
        my_autonomous_system: be_u16 >>
        hold_time: be_u16 >>
        bgp_identifier: be_u32 >>
        optional_parameters_length: be_u8 >>
        (BgpOpenMessage{
            version: version,
            my_autonomous_system: my_autonomous_system,
            hold_time: hold_time,
            bgp_identifier: bgp_identifier,
            optional_parameters_length: optional_parameters_length
        })
    )
);

// Parse BGP Notification message.

#[derive(Debug,PartialEq)]
struct BgpNotificationMessage {
    error_code: u8,
    error_subcode: u8,
    // data (variable)
}

// TODO: Implement validation. And implement handling of data field.
named!(parse_bgp_notification<&[u8], BgpNotificationMessage>,
    do_parse!(
        error_code: be_u8 >>
        error_subcode: be_u8 >>
        (BgpNotificationMessage { error_code: error_code, error_subcode: error_subcode })
    )
);

// Parse BGP Update message.

#[derive(Debug)]
struct BgpUpdateMessage<'a> {
    withdrawn_routes_length: u16,
    withdrawn_routes: Vec<Ipv4Prefix<'a>>, // TODO: make this an Option?
    total_path_attributes_length: u16,
    path_attributes: &'a [u8],
    //nlri: Vec<Ipv4Prefix<'a>> // TODO: make this an Option?
}

// TODO: It's a pain that to calculate one of the field lengths you need
// to know the total message length from the header. Ugh.

named!(parse_bgp_update<&[u8], BgpUpdateMessage>,
    do_parse!(
        withdrawn_routes_length: be_u16 >>
        // TODO: Maybe wrap this in a cond!()?
        withdrawn_routes: flat_map!(take!(withdrawn_routes_length), complete!(many0!(parse_bgp_prefix))) >>

        total_path_attributes_length: be_u16 >>
        path_attributes: take!(total_path_attributes_length) >>
        
        // TODO: This needs to come from the header. For now faking it.
        //total_message_length: value!(29) >>
        //nlri_length: value!(total_message_length - 23 - total_path_attributes_length - withdrawn_routes_length) >>
        //nlri: flat_map!(take!(nlri_length), complete!(many0!(parse_bgp_prefix))) >>
        
        (BgpUpdateMessage{
            withdrawn_routes_length: withdrawn_routes_length,
            withdrawn_routes: withdrawn_routes,
            total_path_attributes_length: total_path_attributes_length,
            path_attributes: path_attributes,
            //nlri: nlri
        })
    )
);

// A BGP prefix found in withdrawn routes and NLRI.
//
// TODO: This is currently not padding out to four octects. And maybe
// this should convert to the Rust Ipv4Addr type.

#[derive(Debug, PartialEq)]
struct Ipv4Prefix<'a> {
    prefix: &'a [u8],
    length: u8,
}

named!(parse_bgp_prefix<&[u8], Ipv4Prefix>,
    do_parse!(
        len_bits: be_u8 >>
        prefix: take!((len_bits + 7) / 8) >>
        (Ipv4Prefix { prefix: prefix, length: len_bits })
    )
);

// Extract the BGP Path Attribute Flags. Is there a nicer way to do
// this?
//
// TODO: Add validation, e.g. transitive must be 1 if optional is 0,
// and the lower 4 bits must be zero.
//
// Rant:
// Why do we even have some of these flags? Think about this. The first
// flag is the optional flag. It defines it the attribute is well knwon
// or optional. If it's well-known, then of course by virtue of it being
// well-known you know that from it's type code. If it's optional, then
// similarly don't you also know it's optional from it's type code by the
// virtue of it either being something your BGP speaker recognizes, of by
// virtue of the fact that it doesn't but your BGP speaker recognizes all
// of the well-known attributes. Quoting the RFC: BGP implementations MUST
// recognize all well-known attributes. The only use for having this flag
// is to be able to distinguish between ANY unrecognized attribute, and a
// unrecognized attribute that you recieve which has the well-known flag
// set. This is a pointless error, so we've added this flag, and a
// pointless error, and made error processing more complex on top of it!
//
// The next bit is the Transitive bit. It is somewhat more useful as it
// tells the BGP speaker what to do with the atribute if it doesn't
// recognize it. When the BGP speaker receives an optional *transitive*
// attribute it doesn't recognize it should pass it on. That's only a
// SHOULD in the RFC. There aren't that many optional transitive
// attributes though, why not just drop them if you don't speak them? I
// mean really, how hard is it to add recognition for a new attribute
// type and determine if you should transit it or not on that basis?
//
// The third bit is the partial bit. It defines whether the information
// contained in the optional transitive attribute is partial. It only
// applies to optional transitive attributes. Which aren't many, and it
// gets set by bgp speakers that don't recognize an optional transitive
// attribute, but decide they'll pass it on anyway. So if you don't do
// that, you don't need this bit.
//
// The fourth bit is the extended length bit, it makes life difficult for
// no reason. See further below.

#[derive(Debug,PartialEq)]
struct BgpPathAttributeFlags {
    optional: bool,
    transitive: bool,
    partial: bool,
    extended_length: bool
}

named!(parse_bgp_path_attribute_flags<&[u8], BgpPathAttributeFlags>,
    do_parse!(
        flags: bits!(tuple!(take_bits!(u8, 1), take_bits!(u8, 1), take_bits!(u8, 1), take_bits!(u8, 1))) >>
        (BgpPathAttributeFlags {
            optional: flags.0 == 1,
            transitive: flags.1 == 1,
            partial: flags.2 == 1,
            extended_length: flags.3 == 1
        })
    )
);

// Extract the regular length of the extended length.
//
// TODO: This is so ugly. What's a nicer way to do this?

fn parse_bgp_path_attribute_length(i: &[u8], extended_length: bool) -> IResult<&[u8], u16> {
    if extended_length == true {
        if i.len() < 2 {
            IResult::Incomplete(Needed::Size(2))
        } else {
            let res = ((i[0] as u16) << 8) + i[1] as u16;
            IResult::Done(&i[2..], res)
        }    
    }
    else {
        if i.len() < 1 {
            IResult::Incomplete(Needed::Size(1))
        } else {
            IResult::Done(&i[1..], i[0] as u16)
        }
    }
}

#[derive(Debug,PartialEq)]
struct BgpPathAttribute<'a> {
    flags: BgpPathAttributeFlags,
    type_code: u8,
    length: u16,
    value: &'a [u8]
}

// Extract BGP Path Attributes.
//
// The length field is either one or two bytes based on the extended
// length bit in the flags. This makes parsing more difficult than it
// should be, just for the sake of savings a few bytes.
//
// There are only four bits used in the flags. The lower four bits of
// the flags field must be empty and ignored. There's four extra bits
// that could've been used right there. Conveniently a 12 bit length
// field would give us lengths up to 4096 bytes, which is more than
// enough given the maximum length of a BGP message is also 4096 bytes.
//
// TODO: Should parsing all attributes be done with a permutation of
// attribute specific parsers? This would be more elegant, but the
// problem is it would require either re-parsing the same flags field
// for each attribute sub-parser, or parsing them once then passing
// the extracted flags to each sub-parser as an argument.
//
// Given that things like flags and length need to be verified as valid
// for each attribute type it seems doing that would be cleaner.

named!(parse_bgp_path_attribute<&[u8], BgpPathAttribute>,
    do_parse!(
        flags: parse_bgp_path_attribute_flags >>
        type_code: be_u8 >>
        // TODO: This is so ugly. What's a nicer way to do this?
        length: call!(parse_bgp_path_attribute_length, flags.extended_length) >>
        value: take!(length) >>
        // TODO: convert to switch here and call the next parser of the
        // content based on the type_code.
        (BgpPathAttribute {
            flags: flags,
            type_code: type_code,
            length: length as u16,
            value: value
        })
    )
);

// Or... would this method below be tidier? It would seem (intuitively)
// to be less efficient.

#[derive(Debug,PartialEq)]
enum PathAttribute {
    Origin(Box<OriginAttribute>),
    AsPath(Box<AsPathAttribute>),
    NextHop(Box<NextHopAttribute>),
    MultiExitDisc(Box<MultiExitDiscAttribute>),
    LocalPref(Box<LocalPrefAttribute>),
    AtomicAggregate,
    Aggregator(Box<AggregatorAttribute>),
}

#[derive(Debug,PartialEq)]
enum BgpOriginCode {
    Igp,
    Egp,
    Incomplete,
}

impl From<u8> for BgpOriginCode {
    fn from(origin_code: u8) -> BgpOriginCode {
        match origin_code {
            0 => BgpOriginCode::Igp,
            1 => BgpOriginCode::Egp,
            2 => BgpOriginCode::Incomplete,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug,PartialEq)]
struct OriginAttribute {
    origin_code: BgpOriginCode,
}

named!(origin_attribute<&[u8], PathAttribute>,
    do_parse!(
        bits!(tag_bits!(u8, 8, 0b0100_0000)) >>
        tag!([1u8]) >> // type code 1
        tag!([1u8]) >> // length should always be 1
        origin_code: be_u8 >>
        (PathAttribute::Origin(Box::new(OriginAttribute { origin_code: BgpOriginCode::from(origin_code) })))
    )
);

#[derive(Debug,PartialEq)]
enum AsPathSegment {
    AsSet(Vec<u16>),
    AsSequence(Vec<u16>),
}

#[derive(Debug,PartialEq)]
struct AsPathAttribute {
    as_path: Vec<AsPathSegment>,
}

named!(as_set<&[u8], Vec<u16>>, preceded!(tag!([1u8]), length_count!(be_u8, be_u16)));
named!(as_sequence<&[u8], Vec<u16>>, preceded!(tag!([2u8]), length_count!(be_u8, be_u16)));
named!(as_path_segment_as_vec1<&[u8], Vec<u16>>, preceded!(alt!(tag!([1u8]) | tag!([2u8])), length_count!(be_u8, be_u16)));
named!(as_path_segment_as_vec2<&[u8], Vec<u16>>, alt!(as_set | as_sequence));

named!(as_path_segment<&[u8], AsPathSegment>,
    do_parse!(
        type_code: verify!(be_u8, |v: u8| v >= 1 && v <= 2) >> // TODO: or use alt!() or one_of!()?
        seg: length_count!(be_u8, be_u16) >>
        (match type_code {
            1u8 => AsPathSegment::AsSet(seg),
            2u8 => AsPathSegment::AsSequence(seg),
            _ => unreachable!(),
        })
    )
);

named!(as_path_attribute<&[u8], PathAttribute>,
    do_parse!(
        bits!(tag_bits!(u8, 8, 0b0100_0000)) >>
        tag!([2u8]) >> // as_path type code is 2
        length: be_u8 >> // TODO: need to recognize extended length flag
        as_path_segments: flat_map!(take!(length), complete!(many0!(as_path_segment))) >>
        (PathAttribute::AsPath(Box::new(AsPathAttribute { as_path: as_path_segments })))
    )
);

#[derive(Debug,PartialEq)]
struct NextHopAttribute {
    next_hop: Ipv4Addr,
}

named!(next_hop_attribute<&[u8], PathAttribute>,
    do_parse!(
        bits!(tag_bits!(u8, 8, 0b0100_0000)) >>
        tag!([3u8]) >> // type code 3
        tag!([4u8]) >> // length should always be 4 (TODO: confirm)
        next_hop: take!(4) >>
        (PathAttribute::NextHop(Box::new(NextHopAttribute { next_hop: Ipv4Addr::new(next_hop[0], next_hop[1], next_hop[2], next_hop[3]) })))
    )
);

#[derive(Debug,PartialEq)]
struct MultiExitDiscAttribute {
    metric: u32,
}

named!(multi_exit_disc_attribute<&[u8], PathAttribute>,
    do_parse!(
        bits!(tag_bits!(u8, 8, 0b1000_0000)) >>
        tag!([4u8]) >> // type code 4
        tag!([4u8]) >> // length should always be 4
        metric: be_u32 >>
        (PathAttribute::MultiExitDisc(Box::new(MultiExitDiscAttribute { metric: metric })))
    )
);

#[derive(Debug,PartialEq)]
struct LocalPrefAttribute {
    preference: u32,
}

named!(local_pref_attribute<&[u8], PathAttribute>,
    do_parse!(
        bits!(tag_bits!(u8, 8, 0b0100_0000)) >>
        tag!([5u8]) >> // type code 5
        tag!([4u8]) >> // length should always be 4
        preference: be_u32 >>
        (PathAttribute::LocalPref(Box::new(LocalPrefAttribute { preference: preference })))
    )
);

#[derive(Debug,PartialEq)]
struct AtomicAggregateAttribute {
}

named!(atomic_aggregate_attribute<&[u8], PathAttribute>,
    do_parse!(
        bits!(tag_bits!(u8, 8, 0b0100_0000)) >>
        tag!([6u8]) >> // type code 5
        tag!([0u8]) >> // length should always be 0
        (PathAttribute::AtomicAggregate)
    )
);

#[derive(Debug,PartialEq)]
struct AggregatorAttribute {
    aggregator_as: u16,
    aggregator_id: Ipv4Addr,
}

named!(aggregator_attribute<&[u8], PathAttribute>,
    do_parse!(
        bits!(tag_bits!(u8, 8, 0b1100_0000)) >>
        tag!([7u8]) >> // type code 5
        tag!([6u8]) >> // length should always be 6
        aggregator_as: be_u16 >>
        aggregator_id: take!(4) >>
        (PathAttribute::Aggregator(Box::new(AggregatorAttribute { aggregator_as: aggregator_as, aggregator_id: Ipv4Addr::new(aggregator_id[0], aggregator_id[1], aggregator_id[2], aggregator_id[3]) })))
    )
);

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{HexDisplay, IResult};

    #[test]
    fn parse_bgp_header_test() {
        let data = include_bytes!("../assets/test_bgp_update1.bin");
        let slice = &data[..];
        assert_eq!(parse_bgp_header(slice), IResult::Done(&slice[19..], BgpMessageHeader { length: 98, message_type: 2 }));
    }

    #[test]
    fn parse_bgp_open_test() {
        let data = include_bytes!("../assets/test_bgp_open1.bin");
        let slice = &data[19..];
        assert_eq!(parse_bgp_open(slice),
            IResult::Done(&slice[10..],
                BgpOpenMessage { version: 4, my_autonomous_system: 65033, hold_time: 180, bgp_identifier: 3232235535, optional_parameters_length: 0 }));
    }

    #[test]
    fn parse_bgp_notification_test() {
        let data = include_bytes!("../assets/test_bgp_notification1.bin");
        let slice = &data[19..];
        assert_eq!(parse_bgp_notification(slice), IResult::Done(&slice[2..], BgpNotificationMessage { error_code: 2, error_subcode: 2 }));
    }

    #[test]
    fn parse_bgp_prefix_test() {
        let data = include_bytes!("../assets/test_bgp_nlri2.bin");
        assert_eq!(parse_bgp_prefix(data), IResult::Done(&b""[..], Ipv4Prefix { prefix: &[192u8, 168, 4], length: 22 }));
    }

    #[test]
    fn parse_bgp_path_attribute_flags_test() {
        assert_eq!(parse_bgp_path_attribute_flags(&[0b10101010]), IResult::Done(&b""[..], BgpPathAttributeFlags { optional: true, transitive: false, partial: true, extended_length: false }));
        assert_eq!(parse_bgp_path_attribute_flags(&[0b11111111]), IResult::Done(&b""[..], BgpPathAttributeFlags { optional: true, transitive: true, partial: true, extended_length: true }));
        assert_eq!(parse_bgp_path_attribute_flags(&[0b01010101]), IResult::Done(&b""[..], BgpPathAttributeFlags { optional: false, transitive: true, partial: false, extended_length: true }));
        assert_eq!(parse_bgp_path_attribute_flags(&[0b11001100]), IResult::Done(&b""[..], BgpPathAttributeFlags { optional: true, transitive: true, partial: false, extended_length: false }));
    }

    #[test]
    fn parse_bgp_path_attribute_length_test() {
        assert_eq!(parse_bgp_path_attribute_length(&[170u8], false), IResult::Done(&b""[..], 170u16));
        assert_eq!(parse_bgp_path_attribute_length(&[111u8], false), IResult::Done(&b""[..], 111u16));
        assert_eq!(parse_bgp_path_attribute_length(&[0u8, 170u8], true), IResult::Done(&b""[..], 170u16));
        assert_eq!(parse_bgp_path_attribute_length(&[170u8, 170u8], true), IResult::Done(&b""[..], 43690u16));
    }

    #[test]
    fn parse_bgp_path_attribute_test() {
        let data = include_bytes!("../assets/test_bgp_path_attributes3.bin");
        let slice = &data[..];
        assert_eq!(
            parse_bgp_path_attribute(slice),
            IResult::Done(
                &slice[4..],
                BgpPathAttribute {
                    flags: BgpPathAttributeFlags { optional: false, transitive: true, partial: false, extended_length: false },
                    type_code: 1,
                    length: 1,
                    value: &[1]
                }
            )
        );
    }

    /*#[test]
    fn parse_bgp_update_test() {
        let data = include_bytes!("../assets/test_bgp_update1.bin");
        //println!("bytes:\n{}", &data.to_hex(8));
        
        match parse_bgp_update(&data[19..]) {
            IResult::Done(i, o) => { println!("Done({:?}, {:?})", i, o); },
            IResult::Incomplete(n) => { println!("Incomplete: {:?}", n); panic!(); },
            IResult::Error(e) => { println!("Error: {:?}", e); panic!(); }
        }
    }*/
    
    #[test]
    fn new_origin_attribute_test() {
        let input = include_bytes!("../assets/test_bgp_path_attribute_origin1.bin");
        let slice = &input[..];
        assert_eq!(origin_attribute(slice), IResult::Done(&b""[..], PathAttribute::Origin(Box::new(OriginAttribute { origin_code: BgpOriginCode::Incomplete }))));
        
        let input = include_bytes!("../assets/test_bgp_path_attribute_origin2.bin");
        let slice = &input[..];
        assert_eq!(origin_attribute(slice), IResult::Done(&b""[..], PathAttribute::Origin(Box::new(OriginAttribute { origin_code: BgpOriginCode::Igp }))));
        
        let input = include_bytes!("../assets/test_bgp_path_attribute_origin3.bin");
        let slice = &input[..];
        assert_eq!(origin_attribute(slice), IResult::Done(&b""[..], PathAttribute::Origin(Box::new(OriginAttribute { origin_code: BgpOriginCode::Egp }))));
    }

    #[test]
    fn new_next_hop_attribute_test() {
        let input = include_bytes!("../assets/test_bgp_path_attribute_next_hop1.bin");
        let slice = &input[..];   
        assert_eq!(next_hop_attribute(slice), IResult::Done(&b""[..], PathAttribute::NextHop(Box::new(NextHopAttribute { next_hop: Ipv4Addr::new(192, 168, 0, 15) }))));
    
        let input = include_bytes!("../assets/test_bgp_path_attribute_next_hop2.bin");
        let slice = &input[..];   
        assert_eq!(next_hop_attribute(slice), IResult::Done(&b""[..], PathAttribute::NextHop(Box::new(NextHopAttribute { next_hop: Ipv4Addr::new(192, 168, 0, 33) }))));
    }

    #[test]
    fn new_multi_exit_disc_attribute_test() {
        let input = include_bytes!("../assets/test_bgp_path_attribute_multi_exit_disc1.bin");
        let slice = &input[..];
        assert_eq!(multi_exit_disc_attribute(slice), IResult::Done(&b""[..], PathAttribute::MultiExitDisc(Box::new(MultiExitDiscAttribute { metric: 0 }))));
    }
    
    #[test]
    fn new_local_pref_attribute_test() {
        let input = include_bytes!("../assets/test_bgp_path_attribute_local_pref1.bin");
        let slice = &input[..];
        assert_eq!(local_pref_attribute(slice), IResult::Done(&b""[..], PathAttribute::LocalPref(Box::new(LocalPrefAttribute { preference: 100 }))));
    }
    
    #[test]
    fn new_atomic_aggregate_attribute_test() {
        let input = include_bytes!("../assets/test_bgp_path_attribute_atomic_aggregate1.bin");
        let slice = &input[..];
        assert_eq!(atomic_aggregate_attribute(slice), IResult::Done(&b""[..], PathAttribute::AtomicAggregate));
    }

    #[test]
    fn new_aggregator_attribute_test() {
        let input = include_bytes!("../assets/test_bgp_path_attribute_aggregator1.bin");
        let slice = &input[..];
        assert_eq!(aggregator_attribute(slice),
            IResult::Done(&b""[..],
                PathAttribute::Aggregator(Box::new(AggregatorAttribute { aggregator_as: 65210, aggregator_id: Ipv4Addr::new(192, 168, 0, 10) }))
            )
        );
    }

    #[test]
    fn new_as_path_attribute_test() {
        let input = include_bytes!("../assets/test_bgp_path_attribute_as_path1.bin");
        let slice = &input[..];
        assert_eq!(
            as_path_attribute(slice),
            IResult::Done(&b""[..],
                PathAttribute::AsPath(Box::new(AsPathAttribute { as_path: vec![AsPathSegment::AsSet(vec![500, 500]), AsPathSegment::AsSequence(vec![65211])] }))
            )
        );
    }
}
