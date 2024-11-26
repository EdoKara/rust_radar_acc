use core::str;
use std::{default, fmt::Error, fs::{self, read, File}, io::{BufRead, Read, Seek}, iter};
use bzip2::bufread;
use std::io::BufReader;

const MESSAGE_RECORD_SIZE: usize = 2432; // number of bytes in a message segment

#[derive(Default, Debug)]
struct VolumeHeaderRaw{
    volumename:[u8; 12], 
    date:[u8; 4], 
    time:[u8; 4], 
    icao:[u8; 4], 
}

impl VolumeHeaderRaw{
    fn new() -> VolumeHeaderRaw{
        VolumeHeaderRaw{
            volumename:[0_u8; 12],
            date: [0_u8; 4],
            time: [0_u8; 4], 
            icao: [0_u8; 4]
        }
    }
}

struct MessageHeaderRaw{
    messagesize:[u8; 2],
    rda_redundant_channel: [u8;1],
    message_type: [u8;1],
    id_seq_no: [u8;2],
    julian_date: [u8;2], // julian date - 2440586.5
    ms_from_midnight: [u8;4],
    n_segments: [u8;2],
    message_segment_no: [u8;2]
}

struct MessageHeader{
    messagesize: i16,
    rda_redundant_channel: i8,
    message_type: i8,
    id_seq_no: i16,
    julian_date: i16,
    ms_from_midnight: i32,
    n_segments: i16,
    message_segment_no: i16
}

impl TryFrom<MessageHeaderRaw> for MessageHeader{
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: MessageHeaderRaw) -> Result<Self, Self::Error>{
        Ok(MessageHeader{
        messagesize: i16::from_be_bytes(value.messagesize),
        rda_redundant_channel: i8::from_be_bytes(value.rda_redundant_channel),
        message_type: i8::from_be_bytes(value.message_type),
        id_seq_no: i16::from_be_bytes(value.id_seq_no),
        julian_date: i16::from_be_bytes(value.julian_date),
        ms_from_midnight: i32::from_be_bytes(value.ms_from_midnight),
        n_segments: i16::from_be_bytes(value.n_segments),
        message_segment_no: i16::from_be_bytes(value.message_segment_no)
        })
    }
}

#[derive(Debug)]
struct VolumeHeader{
    volumename: String,
    date: i32,
    time: i32,
    icao: String
}

impl TryFrom<VolumeHeaderRaw> for VolumeHeader {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: VolumeHeaderRaw) -> Result<Self, Self::Error> {
        Ok(VolumeHeader{
            volumename: str::from_utf8(&value.volumename)?.to_string(),
            date: i32::from_be_bytes(value.date),
            time: i32::from_be_bytes(value.time),
            icao: str::from_utf8(&value.icao)?.to_string()
        }
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let fp = String::from("./data/test");
    let vh_read_res = read_volume_header(fp).unwrap();
    println!("{:?}", vh_read_res);

    let ff = File::open("./data/test").unwrap();
    let mut reader = BufReader::new(ff);
    let mut cw = [0_u8; 4];
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
    let mut msgtype: [u8; 2] = [0;2];
    msgtype.copy_from_slice(&header[12..14]);
    let mgtyp = i16::from_be_bytes(msgtype);

    println!("{:?}",header);
    println!("{:?}",mgtyp);


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

