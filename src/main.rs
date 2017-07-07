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
use nom::{IResult, IError, Needed, ErrorKind};
use nom::IResult::*;

// We have one top level parser that calls each of the message specific
// parsers based on a switch. When required it passes the length field
// from the common header as an argument.
//
// It would be tider and less error prone (IMO) to have one parser for
// each message type that encapsulates everything about that message,
// including it's type code. And then simply use the alt!() combinator
// in the top level parser. But the BGP format makes that impossible,
// without either backtracking or peeking over many bytes. If it were
// the usual type, length, value this wouldn't be the case. Instead it
// is length, type, value.
//
// To give an example, consider error checking in the first top level
// parser that must check first if the type is recognized, then based
// on the type must do type specific checks on the length field that
// came before it.
//
// The complication of passing the length field from the common header
// to some message specific parsers seems to come from optimizing the
// wire format in order to save a couple of bytes. A similar problem
// arises with parsing the path attributes later.
//
// Nom has two types of error handling: simple and verbose. Simple is
// the default and enables a single custom u32 to be returned. We can
// easily encapsulate both the BGP error code and subcodes using this.
// However, in some cases BGP requires the input data that caused the
// error to be reported to the remote end in the Notification message.
// To accomplish this we must use verbose error handling. With careful
// placement of the return_error!() macro this should meet most of our
// needs. However, it is slower, and will not help in the cases where
// BGP expects errors to be silently ignored, but logged. For that we
// will need to look at a separate logging capability.
//
// ...sigh. Nope, there is a problem with this. Again due to how we are
// parsing the length before the type, and then validating in the type,
// we can't return the data because it's been consumed. We will need to
// move to parsing by peeking ahead to the type field, validating that
// then we can properly validate the length field and return that data.
// (I see why everyone writes hand coded BGP parsers.)

// BGP messages.

#[derive(Debug,PartialEq)]
enum BgpMessage {
    Open(Box<BgpOpenMessage>),
    Update(Box<BgpUpdateMessage>),
    Notification(Box<BgpNotificationMessage>),
    Keepalive,
}

#[derive(Debug,PartialEq)]
struct BgpOpenMessage {
    version: u8,
    my_autonomous_system: u16,
    hold_time: u16,
    bgp_identifier: u32,
    optional_parameters_length: u8,
    // TODO: Implement optional parameters.
}

#[derive(Debug,PartialEq)]
struct BgpUpdateMessage {
    withdrawn_routes: Vec<Ipv4Prefix>, // TODO: make this an Option?
    path_attributes: Vec<BgpPathAttribute>,
    //path_attributes: Vec<PathAttribute>,
    nlri: Vec<Ipv4Prefix>, // TODO: make this an Option?
}

#[derive(Debug, PartialEq)]
struct Ipv4Prefix {
    prefix: Vec<u8>,
    length: u8,
}

#[derive(Debug,PartialEq)]
struct BgpNotificationMessage {
    error_code: u8,
    error_subcode: u8,
    // TODO: Implement the data field.
}

// Top level parser to parse all BGP messages.

/* All errors detected while processing the Message Header MUST be
   indicated by sending the NOTIFICATION message with the Error Code
   Message Header Error.  The Error Subcode elaborates on the specific
   nature of the error.

   The expected value of the Marker field of the message header is all
   ones.  If the Marker field of the message header is not as expected,
   then a synchronization error has occurred and the Error Subcode MUST
   be set to Connection Not Synchronized.

   If at least one of the following is true:

      - if the Length field of the message header is less than 19 or
        greater than 4096, or

      - if the Length field of an OPEN message is less than the minimum
        length of the OPEN message, or

      - if the Length field of an UPDATE message is less than the
        minimum length of the UPDATE message, or

      - if the Length field of a KEEPALIVE message is not equal to 19,
        or

      - if the Length field of a NOTIFICATION message is less than the
        minimum length of the NOTIFICATION message,

   then the Error Subcode MUST be set to Bad Message Length.  The Data
   field MUST contain the erroneous Length field.

   If the Type field of the message header is not recognized, then the
   Error Subcode MUST be set to Bad Message Type.  The Data field MUST
   contain the erroneous Type field.*/

// Error codes
const MESSAGE_HEADER_ERROR: u32 = 1;

// Message header error subcodes
const CONNECTION_NOT_SYNCHRONIZED: u32 = 1;
const BAD_MESSAGE_LENGTH: u32 = 2;
const BAD_MESSAGE_TYPE: u32 = 3;

named!(bgp_header_marker, return_error!(ErrorKind::Custom(MESSAGE_HEADER_ERROR << 8 | CONNECTION_NOT_SYNCHRONIZED), tag!([0xff; 16])));
named!(bgp_header_length<&[u8], u16>, verify!(be_u16, |v: u16| v >= 19 && v <= 4096)); // validate per case inside each message
named!(bgp_header_type<&[u8], u8>, verify!(be_u8, |v: u8| v >= 1 && v <= 5)); // or error subcode not recognized

named!(parse_bgp_message<BgpMessage>,
    do_parse!(
        bgp_header_marker >>
        length: return_error!(ErrorKind::Custom(99), bgp_header_length) >>
        message: switch!(bgp_header_type,
            1u8 => call!(parse_bgp_open) |
            2u8 => call!(parse_bgp_update, length) |
            3u8 => call!(parse_bgp_notification, length) |
            //4u8 => value!(BgpMessage::Keepalive) // TODO: need to test for valid length
            4u8 => call!(parse_bgp_keepalive, length)
            // put error handling for unrecognized type here, return error subcode
        ) >>
        (message)
    )
);

// Parse BGP Open message.

// TODO: Implement proper validation.
// TODO: Implement optional parameters.

named!(parse_bgp_open<&[u8], BgpMessage>,
    do_parse!(
        version: verify!(be_u8, |v: u8| v == 4) >>
        my_autonomous_system: be_u16 >>
        hold_time: be_u16 >>
        bgp_identifier: be_u32 >>
        optional_parameters_length: be_u8 >>
        (BgpMessage::Open(
            Box::new(BgpOpenMessage{
                version: version,
                my_autonomous_system: my_autonomous_system,
                hold_time: hold_time,
                bgp_identifier: bgp_identifier,
                optional_parameters_length: optional_parameters_length,
            })
        ))
    )
);

// Parse BGP Notification message.

// TODO: Implement proper validation. And implement proper handling of
// the data field.

named_args!(parse_bgp_notification(length: u16) <BgpMessage>,
    do_parse!(
        error_code: verify!(be_u8, |v: u8| v >= 1 && v <= 6) >>
        // TODO: The possible error_subcodes depend on the error_code.
        error_subcode: verify!(be_u8, |v: u8| v >= 1 && v <= 11) >>
        data: take!(length - 21) >>
        (BgpMessage::Notification(Box::new(BgpNotificationMessage { error_code: error_code, error_subcode: error_subcode })))
    )
);

// Parse BGP Keepalive message.

named_args!(parse_bgp_keepalive(length: u16) <BgpMessage>,
    do_parse!(
        return_error!(ErrorKind::Custom(999), verify!(value!(length), |v: u16| v == 19)) >>
        (BgpMessage::Keepalive)
    )
);

//verify == 19 or header error and Bad Message Length and set notification data field to length
// so what we'd box up a new bgpmessage of a notification and return it now from all our parsers?
// fuuuuuuck
/*
add_return_error!(ErrorKind::Custom(42),
                    do_parse!(
                        tag!("efgh_") >>
                        res: add_return_error!(ErrorKind::Custom(128), tag!("ijklx")) >>
                        (res)
                    )
                )
            )*/

// Parse BGP Update message.

named_args!(parse_bgp_update(length: u16) <BgpMessage>,
    do_parse!(
        withdrawn_routes_length: be_u16 >>
        withdrawn_routes: flat_map!(take!(withdrawn_routes_length), complete!(many0!(parse_bgp_prefix))) >>
        total_path_attributes_length: be_u16 >>
        path_attributes: flat_map!(take!(total_path_attributes_length), complete!(many0!(old_parse_bgp_path_attribute))) >>
        //path_attributes: flat_map!(take!(total_path_attributes_length), complete!(many0!(new_parse_bgp_path_attribute))) >>
        nlri_length: value!(length - 23 - total_path_attributes_length - withdrawn_routes_length) >>
        nlri: flat_map!(take!(nlri_length), complete!(many0!(parse_bgp_prefix))) >>
        (BgpMessage::Update(
            Box::new(BgpUpdateMessage{
                withdrawn_routes: withdrawn_routes,
                path_attributes: path_attributes,
                nlri: nlri
            })
        ))
    )
);

// Parse a BGP prefix found in withdrawn routes and NLRI.

// TODO: This is currently not padding out to four octects. And maybe
// this should convert to the Rust Ipv4Addr type.

named!(parse_bgp_prefix<&[u8], Ipv4Prefix>,
    do_parse!(
        len_bits: be_u8 >>
        prefix: take!((len_bits + 7) / 8) >>
        (Ipv4Prefix { prefix: prefix.to_vec(), length: len_bits })
    )
);

// BGP Path Attributes.

#[derive(Debug,PartialEq)]
struct BgpPathAttribute {
    flags: BgpPathAttributeFlags,
    attribute: PathAttribute,
}

#[derive(Debug,PartialEq)]
struct BgpPathAttributeFlags {
    optional: bool,
    transitive: bool,
    partial: bool,
    extended_length: bool
}

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

#[derive(Debug,PartialEq)]
enum AsPathSegment {
    AsSet(Vec<u16>),
    AsSequence(Vec<u16>),
}

#[derive(Debug,PartialEq)]
struct AsPathAttribute {
    as_path: Vec<AsPathSegment>,
}

#[derive(Debug,PartialEq)]
struct NextHopAttribute {
    next_hop: Ipv4Addr,
}

#[derive(Debug,PartialEq)]
struct MultiExitDiscAttribute {
    metric: u32,
}

#[derive(Debug,PartialEq)]
struct LocalPrefAttribute {
    preference: u32,
}

#[derive(Debug,PartialEq)]
struct AggregatorAttribute {
    aggregator_as: u16,
    aggregator_id: Ipv4Addr,
}

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

// The length field is either one or two bytes based on the extended
// length bit in the flags. This makes parsing more difficult than it
// should be, just for the sake of savings a few bytes.
//
// There are only four bits used in the flags. The lower four bits of
// the flags field must be empty and ignored. There's four extra bits
// that could've been used right there. Conveniently a 12 bit length
// field would give us lengths up to 4096 bytes, which is more than
// enough given the maximum length of a BGP message is also 4096 bytes.

// TODO: Make child parsers aware of extended length. Either we pass the
// flags to the chlid parsers and the child parsers then validate both
// the flags and length according to the standard, or we pass the length
// and do the flags verification in the parent parser.

named!(old_parse_bgp_path_attribute<&[u8], BgpPathAttribute>,
    do_parse!(
        flags: parse_bgp_path_attribute_flags >>
        type_code: be_u8 >>
        /*length: switch!(value!(flags.extended_length as u8),
            1 => call!(be_u16) |
            0 => map!(call!(be_u8), |v: u8| v as u16)
        ) >>*/
        attribute: switch!(value!(type_code),
            1 => call!(origin_attribute) |
            2 => call!(as_path_attribute) |
            3 => call!(next_hop_attribute) |
            4 => call!(multi_exit_disc_attribute) |
            5 => call!(local_pref_attribute) |
            6 => call!(atomic_aggregate_attribute) |
            7 => call!(aggregator_attribute)
        ) >>
        (BgpPathAttribute { flags: flags, attribute: attribute })
    )
);

named!(origin_attribute<&[u8], PathAttribute>,
    do_parse!(
        tag!([1u8]) >> // length should always be 1
        origin_code: be_u8 >>
        (PathAttribute::Origin(Box::new(OriginAttribute { origin_code: BgpOriginCode::from(origin_code) })))
    )
);

//named!(as_set<&[u8], Vec<u16>>, preceded!(tag!([1u8]), length_count!(be_u8, be_u16)));
//named!(as_sequence<&[u8], Vec<u16>>, preceded!(tag!([2u8]), length_count!(be_u8, be_u16)));
//named!(as_path_segment_as_vec1<&[u8], Vec<u16>>, preceded!(alt!(tag!([1u8]) | tag!([2u8])), length_count!(be_u8, be_u16)));
//named!(as_path_segment_as_vec2<&[u8], Vec<u16>>, alt!(as_set | as_sequence));

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
        length: be_u8 >> // TODO: need to recognize extended length flag
        as_path_segments: flat_map!(take!(length), complete!(many0!(as_path_segment))) >>
        (PathAttribute::AsPath(Box::new(AsPathAttribute { as_path: as_path_segments })))
    )
);

named!(next_hop_attribute<&[u8], PathAttribute>,
    do_parse!(
        tag!([4u8]) >> // length should always be 4 (TODO: confirm)
        next_hop: take!(4) >>
        (PathAttribute::NextHop(Box::new(NextHopAttribute { next_hop: Ipv4Addr::new(next_hop[0], next_hop[1], next_hop[2], next_hop[3]) })))
    )
);

named!(multi_exit_disc_attribute<&[u8], PathAttribute>,
    do_parse!(
        tag!([4u8]) >> // length should always be 4
        metric: be_u32 >>
        (PathAttribute::MultiExitDisc(Box::new(MultiExitDiscAttribute { metric: metric })))
    )
);

named!(local_pref_attribute<&[u8], PathAttribute>,
    do_parse!(
        tag!([4u8]) >> // length should always be 4
        preference: be_u32 >>
        (PathAttribute::LocalPref(Box::new(LocalPrefAttribute { preference: preference })))
    )
);

named!(atomic_aggregate_attribute<&[u8], PathAttribute>,
    do_parse!(
        tag!([0u8]) >> // length should always be 0
        (PathAttribute::AtomicAggregate)
    )
);

named!(aggregator_attribute<&[u8], PathAttribute>,
    do_parse!(
        tag!([6u8]) >> // length should always be 6
        aggregator_as: be_u16 >>
        aggregator_id: take!(4) >>
        (PathAttribute::Aggregator(Box::new(AggregatorAttribute { aggregator_as: aggregator_as, aggregator_id: Ipv4Addr::new(aggregator_id[0], aggregator_id[1], aggregator_id[2], aggregator_id[3]) })))
    )
);

// Or... would this method below be tidier? It would seem (intuitively)
// to be less efficient, but it would allow encapsulating all of the
// validation in one place.
//
// This would be the tidier approach if only the flags had been placed
// after the type code.

// TODO: These need to work for cases where different flags are set and
// need to preserve certain optional flags like the partial bit.

named!(new_parse_bgp_path_attribute<&[u8], PathAttribute>,
    alt!(new_origin_attribute | new_as_path_attribute | new_next_hop_attribute | 
        new_multi_exit_disc_attribute | new_local_pref_attribute | new_atomic_aggregate_attribute |
        new_aggregator_attribute)
);

named!(new_origin_attribute<&[u8], PathAttribute>,
    do_parse!(
        bits!(tag_bits!(u8, 8, 0b0100_0000)) >>
        tag!([1u8]) >> // type code 1
        attr: origin_attribute >>
        (attr)
    )
);

named!(new_as_path_attribute<&[u8], PathAttribute>,
    do_parse!(
        bits!(tag_bits!(u8, 8, 0b0100_0000)) >>
        tag!([2u8]) >> // as_path type code is 2
        attr: as_path_attribute >>
        (attr)
    )
);

named!(new_next_hop_attribute<&[u8], PathAttribute>,
    do_parse!(
        bits!(tag_bits!(u8, 8, 0b0100_0000)) >>
        tag!([3u8]) >> // type code 3
        attr: next_hop_attribute >>
        (attr)
    )
);

named!(new_multi_exit_disc_attribute<&[u8], PathAttribute>,
    do_parse!(
        bits!(tag_bits!(u8, 8, 0b1000_0000)) >>
        tag!([4u8]) >> // type code 4
        attr: multi_exit_disc_attribute >>
        (attr)
    )
);

named!(new_local_pref_attribute<&[u8], PathAttribute>,
    do_parse!(
        bits!(tag_bits!(u8, 8, 0b0100_0000)) >>
        tag!([5u8]) >> // type code 5
        attr: local_pref_attribute >>
        (attr)
    )
);

named!(new_atomic_aggregate_attribute<&[u8], PathAttribute>,
    do_parse!(
        bits!(tag_bits!(u8, 8, 0b0100_0000)) >>
        tag!([6u8]) >> // type code 5
        attr: atomic_aggregate_attribute >>
        (attr)
    )
);

named!(new_aggregator_attribute<&[u8], PathAttribute>,
    do_parse!(
        bits!(tag_bits!(u8, 8, 0b1100_0000)) >>
        tag!([7u8]) >> // type code 5
        attr: aggregator_attribute >>
        (attr)
    )
);

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{HexDisplay, IResult};

    #[test]
    fn parse_bgp_open_test() {
        let input = include_bytes!("../assets/test_bgp_open1.bin");
        let slice = &input[19..];
        assert_eq!(parse_bgp_open(slice),
            IResult::Done(&b""[..], BgpMessage::Open(Box::new(BgpOpenMessage { version: 4, my_autonomous_system: 65033, hold_time: 180, bgp_identifier: 3232235535, optional_parameters_length: 0 })))
        );
    }

    #[test]
    fn parse_bgp_open_full_test() {
        let input = include_bytes!("../assets/test_bgp_open1.bin");
        let slice = &input[..];
        assert_eq!(parse_bgp_message(slice),
            IResult::Done(&b""[..], BgpMessage::Open(Box::new(BgpOpenMessage { version: 4, my_autonomous_system: 65033, hold_time: 180, bgp_identifier: 3232235535, optional_parameters_length: 0 })))
        );
    }

    #[test]
    fn parse_bgp_notification_test() {
        let input = include_bytes!("../assets/test_bgp_notification1.bin");
        let slice = &input[19..];
        assert_eq!(parse_bgp_notification(slice, 23), IResult::Done(&b""[..], BgpMessage::Notification(Box::new(BgpNotificationMessage { error_code: 2, error_subcode: 2 }))));
    }

    #[test]
    fn parse_bgp_notification_full_test() {
        let input = include_bytes!("../assets/test_bgp_notification1.bin");
        let slice = &input[..];
        assert_eq!(parse_bgp_message(slice), IResult::Done(&b""[..], BgpMessage::Notification(Box::new(BgpNotificationMessage { error_code: 2, error_subcode: 2 }))));
    }

    #[test]
    fn parse_bgp_keepalive_test() {
        let input = include_bytes!("../assets/test_bgp_keepalive1.bin");
        let slice = &input[..];
        assert_eq!(parse_bgp_message(slice), IResult::Done(&b""[..], BgpMessage::Keepalive));
        
        let x = &mut Vec::from(slice);
        x[17] = 0xFF;
        //assert_eq!(parse_bgp_message(x), Error(ErrorKind::Verify));
        
        match parse_bgp_message(x) {
            IResult::Done(i, o) => { println!("Done({:?}, {:?})", i, o); },
            IResult::Incomplete(n) => { println!("Incomplete: {:?}", n); panic!(); },
            IResult::Error(e) => { println!("Error: {:?}", e); panic!(); }
        }
    }

    #[test]
    fn parse_bgp_update_test() {
        let input = include_bytes!("../assets/test_bgp_update1.bin");
        let slice = &input[19..];
        
        // TODO: fix
        /*match parse_bgp_update(slice, 98) {
            IResult::Done(i, o) => { println!("Done({:?}, {:?})", i, o); },
            IResult::Incomplete(n) => { println!("Incomplete: {:?}", n); panic!(); },
            IResult::Error(e) => { println!("Error: {:?}", e); panic!(); }
        }*/
    }

    #[test]
    fn parse_bgp_update_full_test() {
        let input = include_bytes!("../assets/test_bgp_update1.bin");
        let slice = &input[..];

        // TODO: fix
        /*match parse_bgp_message(slice) {
            IResult::Done(i, o) => { println!("Done({:?}, {:?})", i, o); },
            IResult::Incomplete(n) => { println!("Incomplete: {:?}", n); panic!(); },
            IResult::Error(e) => { println!("Error: {:?}", e); panic!(); }
        }*/
    }

    #[test]
    fn parse_bgp_prefix_test() {
        let input = include_bytes!("../assets/test_bgp_nlri2.bin");
        assert_eq!(parse_bgp_prefix(input), IResult::Done(&b""[..], Ipv4Prefix { prefix: vec![192u8, 168, 4], length: 22 }));
    }

    #[test]
    fn parse_bgp_path_attribute_flags_test() {
        assert_eq!(parse_bgp_path_attribute_flags(&[0b10101010]), IResult::Done(&b""[..], BgpPathAttributeFlags { optional: true, transitive: false, partial: true, extended_length: false }));
        assert_eq!(parse_bgp_path_attribute_flags(&[0b11111111]), IResult::Done(&b""[..], BgpPathAttributeFlags { optional: true, transitive: true, partial: true, extended_length: true }));
        assert_eq!(parse_bgp_path_attribute_flags(&[0b01010101]), IResult::Done(&b""[..], BgpPathAttributeFlags { optional: false, transitive: true, partial: false, extended_length: true }));
        assert_eq!(parse_bgp_path_attribute_flags(&[0b11001100]), IResult::Done(&b""[..], BgpPathAttributeFlags { optional: true, transitive: true, partial: false, extended_length: false }));
    }

    #[test]
    fn parse_bgp_path_attribute_test() {
        let input = include_bytes!("../assets/test_bgp_path_attributes3.bin");
        let slice = &input[..];
        
        assert_eq!(old_parse_bgp_path_attribute(slice),
            IResult::Done(&slice[4..],
                BgpPathAttribute {
                    flags: BgpPathAttributeFlags { optional: false, transitive: true, partial: false, extended_length: false },
                    attribute: PathAttribute::Origin(Box::new(OriginAttribute { origin_code: BgpOriginCode::Egp }))
                }
            )
        );

        assert_eq!(new_parse_bgp_path_attribute(slice),
            IResult::Done(&slice[4..], PathAttribute::Origin(Box::new(OriginAttribute { origin_code: BgpOriginCode::Egp })))
        );

        /*
        let s1 = [0b00000000, 2u8, 4u8, 1u8, 1u8, 0xFF, 0xFF];
        let s2 = [0b00010000, 2u8, 0u8, 4u8, 1u8, 1u8, 0xFF, 0xFF];

        // not extended length
        assert_eq!(parse_bgp_path_attribute(&s1[..]),
            IResult::Done(&b""[..],
                BgpPathAttribute {
                    flags: BgpPathAttributeFlags { optional: false, transitive: false, partial: false, extended_length: false },
                    attribute: ...
                }
            )
        );
        
        // extended length
        assert_eq!(parse_bgp_path_attribute(&s2[..]),
            IResult::Done(&b""[..],
                BgpPathAttribute {
                    flags: BgpPathAttributeFlags { optional: false, transitive: false, partial: false, extended_length: true },
                    attribute: ...
                }
            )
        );*/
    }
    
    #[test]
    fn new_origin_attribute_test() {
        let input = include_bytes!("../assets/test_bgp_path_attribute_origin1.bin");
        let slice = &input[..];
        assert_eq!(new_origin_attribute(slice), IResult::Done(&b""[..], PathAttribute::Origin(Box::new(OriginAttribute { origin_code: BgpOriginCode::Incomplete }))));
        
        let input = include_bytes!("../assets/test_bgp_path_attribute_origin2.bin");
        let slice = &input[..];
        assert_eq!(new_origin_attribute(slice), IResult::Done(&b""[..], PathAttribute::Origin(Box::new(OriginAttribute { origin_code: BgpOriginCode::Igp }))));
        
        let input = include_bytes!("../assets/test_bgp_path_attribute_origin3.bin");
        let slice = &input[..];
        assert_eq!(new_origin_attribute(slice), IResult::Done(&b""[..], PathAttribute::Origin(Box::new(OriginAttribute { origin_code: BgpOriginCode::Egp }))));
    }

    #[test]
    fn new_as_path_attribute_test() {
        let input = include_bytes!("../assets/test_bgp_path_attribute_as_path1.bin");
        let slice = &input[..];
        assert_eq!(new_as_path_attribute(slice), IResult::Done(&b""[..], PathAttribute::AsPath(Box::new(AsPathAttribute { as_path: vec![AsPathSegment::AsSet(vec![500, 500]), AsPathSegment::AsSequence(vec![65211])] }))));
    }

    #[test]
    fn new_next_hop_attribute_test() {
        let input = include_bytes!("../assets/test_bgp_path_attribute_next_hop1.bin");
        let slice = &input[..];   
        assert_eq!(new_next_hop_attribute(slice), IResult::Done(&b""[..], PathAttribute::NextHop(Box::new(NextHopAttribute { next_hop: Ipv4Addr::new(192, 168, 0, 15) }))));
    
        let input = include_bytes!("../assets/test_bgp_path_attribute_next_hop2.bin");
        let slice = &input[..];   
        assert_eq!(new_next_hop_attribute(slice), IResult::Done(&b""[..], PathAttribute::NextHop(Box::new(NextHopAttribute { next_hop: Ipv4Addr::new(192, 168, 0, 33) }))));
    }

    #[test]
    fn new_multi_exit_disc_attribute_test() {
        let input = include_bytes!("../assets/test_bgp_path_attribute_multi_exit_disc1.bin");
        let slice = &input[..];
        assert_eq!(new_multi_exit_disc_attribute(slice), IResult::Done(&b""[..], PathAttribute::MultiExitDisc(Box::new(MultiExitDiscAttribute { metric: 0 }))));
    }
    
    #[test]
    fn new_local_pref_attribute_test() {
        let input = include_bytes!("../assets/test_bgp_path_attribute_local_pref1.bin");
        let slice = &input[..];
        assert_eq!(new_local_pref_attribute(slice), IResult::Done(&b""[..], PathAttribute::LocalPref(Box::new(LocalPrefAttribute { preference: 100 }))));
    }
    
    #[test]
    fn new_atomic_aggregate_attribute_test() {
        let input = include_bytes!("../assets/test_bgp_path_attribute_atomic_aggregate1.bin");
        let slice = &input[..];
        assert_eq!(new_atomic_aggregate_attribute(slice), IResult::Done(&b""[..], PathAttribute::AtomicAggregate));
    }

    #[test]
    fn new_aggregator_attribute_test() {
        let input = include_bytes!("../assets/test_bgp_path_attribute_aggregator1.bin");
        let slice = &input[..];
        assert_eq!(new_aggregator_attribute(slice), IResult::Done(&b""[..], PathAttribute::Aggregator(Box::new(AggregatorAttribute { aggregator_as: 65210, aggregator_id: Ipv4Addr::new(192, 168, 0, 10) }))));
    }

    /*#[test]
    fn new_test() {
        named!(err_test,
            preceded!(
                tag!("abcd_"),
                add_return_error!(ErrorKind::Custom(42),
                    do_parse!(
                        tag!("efgh_") >>
                        res: add_return_error!(ErrorKind::Custom(128), tag!("ijklx")) >>
                        (res)
                    )
                )
            )
        );

        let a = &b"abcd_efgh_ijkl_mnop"[..];

        let res_a = err_test(a);
        println!("{:?}", res_a);
    }*/
}
