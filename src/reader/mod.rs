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

use crate::messages::{
    ClutterFilterMapMetadata, DigitalRadarDataGenericFormatHeader,
    DigitalRadarDataGenericFormatHeaderRaw, MessageHeader, MessageHeaderRaw,
    RawClutterFilterMapMetadata, VolumeHeader, VolumeHeaderRaw,
    DIGITAL_RADAR_DATA_GENERIC_FORMAT_HEADER_SIZE,
};

const MESSAGE_RECORD_SIZE: usize = 2432; // number of bytes in a message segment (compressed)
const MESSAGE_HEADER_STARTING_BYTE_OFFSET: usize = 12;
const CONTROL_WORD_SIZE: usize = 4;
const VOLUME_HEADER_SIZE: usize = 24;
const MESSAGE_HEADER_SIZE: usize = 16;

use bzip2::bufread::BzDecoder;
use std::io::SeekFrom;

pub fn segment_message<'a>(ff: File) -> anyhow::Result<Vec<BufReader<&'a std::fs::File>>> {
    // let mut ff: std::fs::File = std::fs::File::open(fp).expect("Failed to open file");

    let mut fileref = &ff;

    let metadata = ff.metadata()?;
    let filesize = metadata.len();
    println!("File size: {} bytes", filesize);
    let mut cursor_position: u64 = 0;

    let mut readers: Vec<bzip2::read::BzDecoder<std::io::Take<&std::fs::File>>> = Vec::new();

    //skips the volume header
    fileref.seek(SeekFrom::Start(24))?;
    cursor_position += 24;

    while cursor_position < filesize {
        println!("Cursor position: {}", cursor_position);
        let mut buf = [0_u8; 4];
        fileref.read_exact(&mut buf)?;
        let control_word = i32::from_be_bytes(buf);
        println!("Control word: {}", control_word);
        let chunksize: u64 = u64::try_from(control_word)
            .expect("If this cast fails the control word is wrong, not the program.");
        println!("chunk size: {}", chunksize);
        let mut decoder = bzip2::read::BzDecoder::new(fileref.take(chunksize));
        readers.push(decoder);
        cursor_position += chunksize;
    }

    let mut processed_bufs: Vec<Vec<u8>> = Vec::new();

    let rs: Vec<_> = readers
        .iter_mut()
        .zip(processed_bufs)
        .map(|(a, mut b)| a.read_to_end(&mut b))
        .collect();

    println!("Total segments: {}", readers.len());

    todo!()
}

pub fn read_volume_header(fp: &str) -> anyhow::Result<VolumeHeader> {
    let file = std::fs::File::open(fp).unwrap();
    let mut reader = BufReader::new(file);
    let mut vh = VolumeHeaderRaw::new();

    reader.read_exact(&mut vh.volumename)?;
    reader.read_exact(&mut vh.date)?;
    reader.read_exact(&mut vh.time)?;
    reader.read_exact(&mut vh.icao)?;

    let vol_header = VolumeHeader::try_from(vh)
        .map_err(|e| anyhow::anyhow!("Failed to convert VolumeHeaderRaw to VolumeHeader: {}", e))?;

    Ok(vol_header)
}

pub fn read_message_header(message: Vec<u8>) -> anyhow::Result<MessageHeader> {
    let (_, message) = message.split_at(MESSAGE_HEADER_STARTING_BYTE_OFFSET);
    let (header, _) = message.split_at(MESSAGE_HEADER_SIZE);

    let mut reader = BufReader::new(header);

    let mut mh = MessageHeaderRaw::new();

    reader.read_exact(&mut mh.messagesize)?;
    reader.read_exact(&mut mh.rda_redundant_channel)?;
    reader.read_exact(&mut mh.message_type)?;
    reader.read_exact(&mut mh.id_seq_no)?;
    reader.read_exact(&mut mh.julian_date)?;
    reader.read_exact(&mut mh.ms_from_midnight)?;
    reader.read_exact(&mut mh.n_segments)?;
    reader.read_exact(&mut mh.message_segment_no)?;

    let message_header = MessageHeader::try_from(mh)
        .map_err(|e| anyhow::anyhow!("Failed to convert VolumeHeaderRaw to VolumeHeader: {}", e))?;

    Ok(message_header)
}

pub fn decompress_nexrad_file(fp: &str) -> anyhow::Result<Vec<Vec<u8>>> {
    let mut ff: std::fs::File = std::fs::File::open(&fp).expect("Failed to open file");
    let mut buf: Vec<u8> = Vec::new();
    let file_length = ff.metadata()?.len();

    let mut position_state: usize = VOLUME_HEADER_SIZE + CONTROL_WORD_SIZE;
    ff.seek(std::io::SeekFrom::Start(position_state as u64))?;

    let mut bufs: Vec<Vec<u8>> = Vec::new();

    loop {
        let mut opbuf: Vec<u8> = Vec::new();
        let mut decoder = bzip2::read::BzDecoder::new(&ff);
        decoder.read_to_end(&mut opbuf)?;

        bufs.push(opbuf);

        position_state += decoder.total_in() as usize;
        position_state += CONTROL_WORD_SIZE;
        ff.seek(std::io::SeekFrom::Start(position_state as u64))?;

        if position_state >= file_length as usize {
            break;
        }
    }

    Ok(bufs)
}

pub fn read_data_header(message: &Vec<u8>) -> anyhow::Result<DigitalRadarDataGenericFormatHeader> {
    let mut dhdr: DigitalRadarDataGenericFormatHeaderRaw =
        DigitalRadarDataGenericFormatHeaderRaw::default();

    let (_, msg) = message.split_at(MESSAGE_HEADER_STARTING_BYTE_OFFSET + MESSAGE_HEADER_SIZE);
    let (header, _) = msg.split_at(DIGITAL_RADAR_DATA_GENERIC_FORMAT_HEADER_SIZE);
    let mut reader = std::io::Cursor::new(header);

    let _ = reader.read_exact(&mut dhdr.radar_identifier);
    let _ = reader.read_exact(&mut dhdr.collection_time);
    let _ = reader.read_exact(&mut dhdr.modified_julian_date);
    let _ = reader.read_exact(&mut dhdr.azimuth_number);
    let _ = reader.read_exact(&mut dhdr.azimuth_angle);
    let _ = reader.read_exact(&mut dhdr.compression_indicator);
    let _ = reader.read_exact(&mut dhdr.spare_byte);
    let _ = reader.read_exact(&mut dhdr.radial_length);
    let _ = reader.read_exact(&mut dhdr.az_res_spacing);
    let _ = reader.read_exact(&mut dhdr.radial_status);
    let _ = reader.read_exact(&mut dhdr.elevation_number);
    let _ = reader.read_exact(&mut dhdr.cut_sector_number);
    let _ = reader.read_exact(&mut dhdr.elevation_angle);
    let _ = reader.read_exact(&mut dhdr.radial_spot_blanking_status);
    let _ = reader.read_exact(&mut dhdr.azimuth_indexing_mode);
    let _ = reader.read_exact(&mut dhdr.data_block_count);
    let _ = reader.read_exact(&mut dhdr.const_vol_data_block_pointer);
    let _ = reader.read_exact(&mut dhdr.const_elevation_data_block_pointer);
    let _ = reader.read_exact(&mut dhdr.const_radial_data_block_pointer);
    let _ = reader.read_exact(&mut dhdr.reflectivity_block_pointer);
    let _ = reader.read_exact(&mut dhdr.velocity_block_pointer);
    let _ = reader.read_exact(&mut dhdr.sw_block_pointer);
    let _ = reader.read_exact(&mut dhdr.diff_ref_block_pointer);
    let _ = reader.read_exact(&mut dhdr.phi_block_pointer);
    let _ = reader.read_exact(&mut dhdr.rho_block_pointer);

    let data_header = DigitalRadarDataGenericFormatHeader::try_from(dhdr).map_err(|e| {
        anyhow::anyhow!(
            "Failed to convert DigitalRadarDataGenericFormatHeaderRaw: {}",
            e
        )
    })?;

    Ok(data_header)
}
