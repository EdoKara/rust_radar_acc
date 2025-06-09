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
use crate::messages::{MessageHeader, MessageHeaderRaw, VolumeHeader, VolumeHeaderRaw, RawClutterFilterMapMetadata, 
ClutterFilterMapMetadata};

const MESSAGE_RECORD_SIZE: usize = 2432; // number of bytes in a message segment (compressed)
const MESSAGE_HEADER_STARTING_BYTE_OFFSET: usize = 12;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fp = String::from("./data/test");
    let vh_read_res = read_volume_header(fp).unwrap();
    println!("{:?}", vh_read_res);

    let ff = File::open("./data/test").unwrap();
    let mut reader = BufReader::new(ff);
    let mut cw = [0_0_u8; 4];
    let mut encoding = [0_u8; 3];
    let _ = reader.seek(std::io::SeekFrom::Start(24))?;
    reader.read_exact(&mut cw)?;
    reader.read_exact(&mut encoding)?;
    reader.seek(std::io::SeekFrom::Current(-3))?;

    let control_word = i32::from_be_bytes(cw);
    let enc = String::from_utf8_lossy(&encoding);
    let t = u64::try_from(control_word).unwrap();

    let mut decoder = bzip2::bufread::BzDecoder::new(reader.take(t));
    let mut metadata_record_entry = Vec::new();
    let _ = decoder.read_to_end(&mut metadata_record_entry);

    // let _ = decoder.read_vectored(&mut metadata_record_entry);

    println!("Size: {}", control_word);
    println!("");
    println!("Encoding: {}", enc);

    // println!("{:?}",String::from_utf8_lossy(metadata_record_entry.as_slice()));
    println!("{:?} Bytes", metadata_record_entry.len());

    let mut header: [u8; 96] = [0; 96];
    header.copy_from_slice(&metadata_record_entry[0..96]);

    // there are 15 bytes in this header struct but 
    // the first 12 are zeros (i.e. they offest it.)

    let mut mhdr = messages::MessageHeaderRaw::new();
    mhdr.messagesize.copy_from_slice(&header[12..14]);
    mhdr.rda_redundant_channel = header.get(14).unwrap().clone();
    mhdr.message_type = header.get(15).unwrap().clone();
    mhdr.id_seq_no.copy_from_slice(&header[16..18]);
    mhdr.julian_date.copy_from_slice(&header[18..20]);
    mhdr.ms_from_midnight.copy_from_slice(&header[20..24]);
    mhdr.n_segments.copy_from_slice(&header[24..26]);
    mhdr.message_segment_no.copy_from_slice(&header[26..28]);

    // next comes the metadata for the actual message record.

    let mhdr_proc = MessageHeader::try_from(mhdr).unwrap();

    let mut cfm_raw: messages::RawClutterFilterMapMetadata = RawClutterFilterMapMetadata::new();

    cfm_raw.map_generation_date.copy_from_slice(&header[28..30]);
    cfm_raw.map_generation_time.copy_from_slice(&header[30..32]);
    cfm_raw.num_elevation_segments.copy_from_slice(&header[32..34]);
    cfm_raw.elevation_segments[0].azimuth_segments[0].num_rangezones.copy_from_slice(&header[34..36]);
    

    println!("Messsage Header: {:?}", header);
    println!("Processed Message Header: {:?}", mhdr_proc);
    println!("Clutter filter map date:{:?}", i16::from_be_bytes(cfm_raw.map_generation_date));
    println!("map generation time: {:?}", i16::from_be_bytes(cfm_raw.map_generation_time));
    println!("number elevation segments{:?}", i16::from_be_bytes(cfm_raw.num_elevation_segments));
    println!("number of range zones (elev 0 az 0){:?}", i16::from_be_bytes(cfm_raw.elevation_segments[0].azimuth_segments[0].num_rangezones));

    Ok(())
}

fn read_volume_header(fp: String) -> Result<VolumeHeader, Box<dyn std::error::Error>> {
    let file = fs::File::open(fp).unwrap();
    let mut reader = BufReader::new(file);
    let mut vh = VolumeHeaderRaw::new();

    reader.read_exact(&mut vh.volumename)?;
    reader.read_exact(&mut vh.date)?;
    reader.read_exact(&mut vh.time)?;
    reader.read_exact(&mut vh.icao)?;

    let vol_header = VolumeHeader::try_from(vh)?;

    Ok(vol_header)
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
