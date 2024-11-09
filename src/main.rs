use core::str;
use std::{default, fmt::Error, fs::{self, read, File}, io::{BufRead, Read, Seek}, iter};
use bzip2::bufread;
use std::io::BufReader;

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

fn main() {
    let file = fs::File::open("./data/test").unwrap();
    let mut reader = BufReader::new(file);
    let mut buf = [0_u8; 12];
    let mut dttest = [0_u8; 4];
    let mut timetest = [0_u8; 4];
    let mut icaobytes = [0_u8; 4];
    let mut control_word = [0_u8; 4];

    // the reader automatically advances
    reader.read_exact(&mut buf);
    reader.read_exact(&mut dttest);
    reader.read_exact(&mut timetest);
    reader.read_exact(&mut icaobytes);
    reader.read_exact(&mut control_word);


    let teststr = String::from_utf8_lossy(&buf);
    let tint = i32::from_be_bytes(dttest);
    let ttest = i32::from_be_bytes(timetest);
    let icao = String::from_utf8_lossy(&icaobytes);
    let ctrlwrd = i32::from_be_bytes(control_word);
    println!("{}",buf.len());

    println!("{:?}", teststr);
    println!("{:?}", tint);
    println!("{:?}", ttest);
    println!("{:?}", icao);
    println!("{:?}", ctrlwrd);

    // let ff = File::open("./data/test").unwrap();
    // let mut br = BufReader::new(ff);

    // let mut dc = bzip2::read::BzDecoder::new(br);

    // let mut output = String::new();

    // dc.read_to_string(& mut output).unwrap();

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

