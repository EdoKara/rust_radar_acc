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

    let mut decoder = bzip2::bufread::BzDecoder::new(reader);

    let control_word = i32::from_be_bytes(cw);
    let enc = String::from_utf8_lossy(&encoding);

    let t = usize::try_from(control_word).unwrap();

    let mut clutter_map_data = [0_u8; 2269];

    
    let _ = decoder.read_exact(&mut clutter_map_data)?;

    println!("Size: {}", control_word);
    println!("");
    println!("Encoding: {}", enc);

    println!("{}",String::from_utf8_lossy(&clutter_map_data));

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

