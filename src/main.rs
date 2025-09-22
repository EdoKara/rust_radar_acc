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
    ClutterFilterMapMetadata, DigitalRadarDataGenericFormat, MessageHeader, MessageHeaderRaw,
    MetadataBlockType, RawClutterFilterMapMetadata, VolumeHeader, VolumeHeaderRaw,
    DIGITAL_RADAR_DATA_GENERIC_FORMAT_HEADER_SIZE, MESSAGE_HEADER_SIZE,
};
use crate::reader::process_metadata_record::{
    process_populated_metadata_blocks, segment_metadata_record,
};
use crate::reader::process_regular_record::{process_regular_blocks, segment_regular_messages};
use crate::reader::{
    decompress_nexrad_file, read_data_header, read_generic_data_headers, read_message_header,
    read_message_headers, read_volume_header,
};

const MESSAGE_RECORD_SIZE: usize = 2432; // number of bytes in a message segment (compressed)
const MESSAGE_HEADER_STARTING_BYTE_OFFSET: usize = 12;
const CONTROL_WORD_SIZE: usize = 4;
const VOLUME_HEADER_SIZE: usize = 24;
fn main() -> anyhow::Result<()> {
    let fp = "./data/test";
    let fp2 = "./data/test2";

    let vh = read_volume_header(&fp)?;

    let segments = decompress_nexrad_file::decompress_nexrad_file(&fp)?;

    println!("Volume Header: {:?}", vh);
    println!("Total segments: {}", segments.len());

    segments.iter().enumerate().for_each(|(index, seg)| {
        println!("{index} Segment type: {:?}", seg.record_type);
        println!("{index} Segment size: {} bytes", seg.buf.len());
    });

    let md_segments = segment_metadata_record(segments[0].clone())?;
    println!("Metadata Record Messages: {}", md_segments.len());
    println!(
        "Populated Metadata Record Messages: {}",
        md_segments
            .iter()
            .filter(|msg| msg.blocktype != MetadataBlockType::Empty)
            .collect::<Vec<_>>()
            .len()
    );

    let pop_md_blocks = process_populated_metadata_blocks(md_segments)?;
    println!("Populated Metadata Messages: {}", pop_md_blocks.len());

    pop_md_blocks.iter().enumerate().for_each(|(index, msg)| {
        println!("Message {index}: {:?}", msg.header);
    });

    let reg_segment = segment_regular_messages(segments[1].clone())?;
    println!("Regular Messages: {}", reg_segment.len());

    reg_segment.iter().enumerate().for_each(|(index, msg)| {
        println!("Message {index}: {:?}", msg.header);
    });

    let x = process_regular_blocks(reg_segment)?;
    println!("Processed Regular Blocks: {}", x.len());

    x.iter().enumerate().for_each(|(index, msg)| {
        println!("Processed Block {index}: {:?}", msg);
    });

    let x: Vec<Vec<DigitalRadarDataGenericFormat>> = segments[1..]
        .to_vec()
        .iter()
        .map(|seg| segment_regular_messages(seg.to_owned()).unwrap())
        .map(|msg| process_regular_blocks(msg).unwrap())
        .collect();

    println!("Processed Regular Blocks from all segments: {}", x.len());

    // for (index, mhdr) in mhdrs.iter().enumerate() {
    //     println!("Message Header {index}: {:?}", mhdr);
    // }
    // let tseg = segments[1].clone();
    // let gdfs = read_generic_data_headers(&tseg)?;
    // println!("Generic Data Format Header: {:?}", gdfs);

    // let dhdrs: Vec<_> = segments
    //     .iter()
    //     .skip(1)
    //     .enumerate()
    //     .map(|(i, seg)| {
    //         println!("{:?}", i);
    //         read_generic_data_headers(seg).unwrap()
    //     })
    //     .collect();
    // let dhdrs2: Vec<_> = segments2
    //     .iter()
    //     .skip(1)
    //     .map(|seg| read_generic_data_headers(seg).unwrap())
    //     .collect();

    // for (index, dhdr) in dhdrs.iter().enumerate() {
    //     println!("Data Header {index}: {:?}", dhdr);
    // }
    // for (index, dhdr) in dhdrs2.iter().enumerate() {
    //     println!("Data Header {index}: {:?}", dhdr);
    // }

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
