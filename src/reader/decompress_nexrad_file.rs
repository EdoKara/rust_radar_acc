use super::{CONTROL_WORD_SIZE, VOLUME_HEADER_SIZE};
use crate::messages::{CompressedRecord, RecordType};

use bzip2::read::BzDecoder;
use std::io::{Read, Seek};

pub fn decompress_nexrad_file(fp: &str) -> anyhow::Result<Vec<CompressedRecord>> {
    let mut ff: std::fs::File = std::fs::File::open(&fp).expect("Failed to open file");
    let mut buf: Vec<u8> = Vec::new();
    let file_length = ff.metadata()?.len();

    let mut position_state: usize = VOLUME_HEADER_SIZE + CONTROL_WORD_SIZE;
    ff.seek(std::io::SeekFrom::Start(position_state as u64))?;

    let mut bufs: Vec<Vec<u8>> = Vec::new();

    loop {
        let mut opbuf: Vec<u8> = Vec::new();
        let mut decoder = BzDecoder::new(&ff);
        decoder.read_to_end(&mut opbuf)?;

        bufs.push(opbuf);

        position_state += decoder.total_in() as usize;
        position_state += CONTROL_WORD_SIZE;
        ff.seek(std::io::SeekFrom::Start(position_state as u64))?;

        if position_state >= file_length as usize {
            break;
        }
    }

    let mut opvec: Vec<CompressedRecord> = Vec::new();

    let metdata_buf = bufs
        .first()
        .expect("there should always be at least one buffer, or the file is corrupted.")
        .to_owned();

    let metadata: CompressedRecord = CompressedRecord {
        record_type: RecordType::MetadataRecord,
        buf: metdata_buf,
    };
    opvec.push(metadata);

    for buf in bufs.iter().skip(1) {
        let record = CompressedRecord {
            record_type: RecordType::NormalRecord,
            buf: buf.to_owned(),
        };
        opvec.push(record);
    }

    Ok(opvec)
}
