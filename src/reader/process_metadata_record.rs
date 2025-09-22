// for processing metadata records specifically.
// These have a strict format that we can use.

// each message in the record is 2432 bytes, and there are 134 messages.
// a message CAN be all zeros if it's not used.

// to do this , we want to break up the metadata record into a series of
// "messages" and then check that they're not empty.
use crate::{
    messages::{
        CompressedRecord, MessageHeader, MessageHeaderRaw, MetadataBlockType, MetadataMessage,
        MetadataMessageBlock,
    },
    reader::read_message_header,
    MESSAGE_HEADER_SIZE, MESSAGE_HEADER_STARTING_BYTE_OFFSET, MESSAGE_RECORD_SIZE,
};
use std::io::{BufReader, Read};

pub fn segment_metadata_record(
    record: CompressedRecord,
) -> anyhow::Result<Vec<MetadataMessageBlock>> {
    Ok(record
        .buf
        .chunks(MESSAGE_RECORD_SIZE)
        .map(|chunk| {
            if chunk.iter().all(|&b| b == 0) {
                MetadataMessageBlock {
                    blocktype: MetadataBlockType::Empty,
                    data: chunk.to_vec(),
                }
            } else {
                MetadataMessageBlock {
                    blocktype: MetadataBlockType::Populated,
                    data: chunk.to_vec(),
                }
            }
        })
        .collect())
}

// now that we have empty and populated segments, we can process the ones that have data in them.

pub fn process_populated_metadata_blocks(
    blocks: Vec<MetadataMessageBlock>,
) -> anyhow::Result<Vec<MetadataMessage>> {
    let mut processed_blocks = Vec::new();

    for block in blocks {
        if block.blocktype == MetadataBlockType::Populated {
            let (_, blk) = block.data.split_at(MESSAGE_HEADER_STARTING_BYTE_OFFSET);
            let (header_bytes, data_bytes) = blk.split_at(MESSAGE_HEADER_SIZE);

            let mhdr = read_message_header(header_bytes.to_vec())?;
            processed_blocks.push(MetadataMessage {
                header: mhdr,
                data: data_bytes.to_vec(),
            })
        }
    }

    Ok(processed_blocks)
}
