use core::str;
use std::{default, fmt::Error, fs::{self, read, File}, io::{BufRead, Read, Seek}, iter};
use bzip2::bufread;
use std::io::BufReader;

pub mod messages;
use crate::messages::{VolumeHeader, VolumeHeaderRaw, MessageHeader, MessageHeaderRaw};

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

    let mut header: [u8; 96] = [0;96];
    header.copy_from_slice(&metadata_record_entry[0..96]);
    println!("{:?}",header);
    println!("{:?}", &header[12..14]);
    println!("{:?}", header.get(14));
    println!("{:?}", header.get(15));
    println!("{:?}", &header[16..18]);
    println!("{:?}", &header[18..20]);
    println!("{:?}", &header[20..24]);
    println!("{:?}", &header[24..26]);
    println!("{:?}", &header[26..28]);

    let mut mhdr = messages::MessageHeaderRaw::new();
    mhdr.messagesize.copy_from_slice(&header[12..14]);
    mhdr.rda_redundant_channel = header.get(14).unwrap().clone();
    mhdr.message_type = header.get(15).unwrap().clone();
    mhdr.id_seq_no.copy_from_slice(&header[16..18]);
    mhdr.julian_date.copy_from_slice(&header[18..20]);
    mhdr.ms_from_midnight.copy_from_slice(&header[20..24]);
    mhdr.n_segments.copy_from_slice(&header[24..26]);
    mhdr.message_segment_no.copy_from_slice(&header[26..28]);
    
    let mhdr_proc = MessageHeader::try_from(mhdr).unwrap();

    
    println!("{:?}",header);
    println!("{:?}", mhdr_proc);


    Ok(())

}

fn read_volume_header(fp: String) -> Result<VolumeHeader, Box<dyn std::error::Error>>{
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

