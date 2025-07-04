use anyhow::Ok;
use bzip2::bufread;
use core::str;
use std::io::BufReader;
use std::{
    default,
    fmt::Error,
    fs::{self, read, File},
    io::{BufRead, Read, Seek},
    iter,
};

pub mod messages;
pub mod reader;
use crate::messages::{
    ClutterFilterMapMetadata, MessageHeader, MessageHeaderRaw, RawClutterFilterMapMetadata,
    VolumeHeader, VolumeHeaderRaw, DIGITAL_RADAR_DATA_GENERIC_FORMAT_HEADER_SIZE,
    MESSAGE_HEADER_SIZE,
};
use crate::reader::{
    decompress_nexrad_file, read_data_header, read_message_header, read_volume_header,
};

const MESSAGE_RECORD_SIZE: usize = 2432; // number of bytes in a message segment (compressed)
const MESSAGE_HEADER_STARTING_BYTE_OFFSET: usize = 12;
const CONTROL_WORD_SIZE: usize = 4;
const VOLUME_HEADER_SIZE: usize = 24;
fn main() -> anyhow::Result<()> {
    let fp = "./data/test";

    let vh = read_volume_header(&fp)?;

    let segments: Vec<Vec<u8>> = decompress_nexrad_file(&fp)?;

    println!("Volume Header: {:?}", vh);
    println!("Total segments: {}", segments.len());

    let mhdrs: Vec<MessageHeader> = segments
        .iter()
        .map(|seg| read_message_header(seg.clone().to_owned()).unwrap())
        .collect();

    let dhdrs: Vec<_> = segments
        .iter()
        .skip(1)
        .map(|seg| read_data_header(seg).unwrap())
        .collect();

    println!("{:?}", dhdrs);
    println!("total data headers: {:?}", dhdrs.len());
    println!("dh 1: {:?}", dhdrs.get(0).unwrap());
    Ok(())
}

// Ok: Each subsequent section has a control word associated with it.
// So after reading a segment we should seek through to each section's control word.
//
// Maybe some way to keep track of the state of the reader's position, in terms of the byte
// offfset?

// Idea:
// for immplementing the recursive read:
// get the number of subsegments from the initialized struct.
// We _should_ know the number of byte offsets within each subsegment unless
// it's recursively variable, but then we just do it again until we get to
// the leaf nodes of the array.
//
// For example, we know the number of azimuths, but not the number
// of ranges that are in each one. For the defined number of ranges,
// we know how big each data packet is for the leaf node. That tells us
// what the offsets are and thus where we can read each packet.
//
// alternatively, we could use the number of subsegments to recursively set up
// read operations for sub-segments of the data file, which would allow a
// static read function that's more general-purpose and have dynamic preparation
// that encapsulates it.

//fn traverse_read_tree(initial_struct: T){}
