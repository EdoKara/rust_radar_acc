// segmenting regular records into messages
//
use crate::messages::{
    CompressedRecord, DigitalRadarDataGenericFormat, DigitalRadarDataGenericFormatBlock,
    DigitalRadarDataGenericFormatBlockRaw, DigitalRadarDataGenericFormatHeader,
    ElevationConstantData, ElevationConstantDataRaw, MessageHeader, MessageHeaderRaw,
    RadialConstantData, RadialConstantDataRaw, RegularMessageBlock, VolumeConstantData,
    VolumeConstantDataRaw, HALFWORD_SIZE,
};
use crate::reader::{
    read_data_header, read_message_header, MESSAGE_HEADER_SIZE, MESSAGE_HEADER_STARTING_BYTE_OFFSET,
};
use std::io::{BufReader, Read, Seek, SeekFrom};

pub fn segment_regular_messages(
    message: CompressedRecord,
) -> anyhow::Result<Vec<RegularMessageBlock>> {
    let mut messages: Vec<RegularMessageBlock> = Vec::new();

    let msglen = message.buf.len();

    let mut reader = std::io::Cursor::new(message.buf);
    reader.seek(std::io::SeekFrom::Start(
        MESSAGE_HEADER_STARTING_BYTE_OFFSET as u64,
    ))?;

    while reader.position() < msglen as u64 {
        let mut header_bytes = vec![0_u8; MESSAGE_HEADER_SIZE];
        reader.read_exact(&mut header_bytes)?;

        let header = read_message_header(header_bytes)?;
        let data_size = ((header.messagesize * HALFWORD_SIZE as i16) as usize
            - MESSAGE_HEADER_SIZE)
            + MESSAGE_HEADER_STARTING_BYTE_OFFSET;
        let mut data_bytes = vec![0_u8; data_size];

        let _ = reader.read(&mut data_bytes)?;

        messages.push(RegularMessageBlock {
            header,
            data: data_bytes,
        });
    }

    Ok(messages)
}

pub fn process_regular_blocks(
    blocks: Vec<RegularMessageBlock>,
) -> anyhow::Result<Vec<DigitalRadarDataGenericFormat>> {
    let blocks: Vec<DigitalRadarDataGenericFormat> = blocks
        .iter()
        .map(|block| {
            let data = block.data.clone();

            let header = read_data_header(&data.as_slice()).unwrap();
            println!("Header: {:?}", header);
            let volume_data =
                read_volume_constant_data(&data.as_slice(), header.const_vol_data_block_pointer);

            let elev_const_data = read_elevation_constant_data(
                &data.as_slice(),
                header.const_elevation_data_block_pointer,
            );

            let radial_const_data =
                read_radial_constant_data(&data.as_slice(), header.const_radial_data_block_pointer);

            let reflectivity_block =
                read_format_moment_block(&data.as_slice(), header.reflectivity_block_pointer);
            let velocity_block =
                read_format_moment_block(&data.as_slice(), header.velocity_block_pointer);
            let spectrum_width_block =
                read_format_moment_block(&data.as_slice(), header.sw_block_pointer);
            let diff_ref_block =
                read_format_moment_block(&data.as_slice(), header.diff_ref_block_pointer);
            let phi_block = read_format_moment_block(&data.as_slice(), header.phi_block_pointer);
            let rho_block = read_format_moment_block(&data.as_slice(), header.rho_block_pointer);
            let moments = vec![
                reflectivity_block,
                velocity_block,
                spectrum_width_block,
                diff_ref_block,
                phi_block,
                rho_block,
            ];

            DigitalRadarDataGenericFormat {
                header: header,
                data: data,
                volume_constant_data: volume_data,
                elevation_constant_data: elev_const_data,
                radial_constant_data: radial_const_data,
                moment_blocks: moments,
            }
        })
        .collect();

    Ok(blocks)
}

fn read_volume_constant_data(data: &[u8], pointer: i32) -> Option<VolumeConstantData> {
    match pointer {
        0 => return None, // no data to read
        _ => {
            let mut reader = std::io::Cursor::new(data);

            // get to the block offset.
            reader.seek(SeekFrom::Start(pointer as u64)).ok()?;

            let mut raw_data = VolumeConstantDataRaw::default();

            let _ = reader.read_exact(&mut raw_data.block_type);
            let _ = reader.read_exact(&mut raw_data.moment_name);
            let _ = reader.read_exact(&mut raw_data.lrtup);
            let _ = reader.read_exact(&mut raw_data.version_number);
            let _ = reader.read_exact(&mut raw_data.version_Number_minor);
            let _ = reader.read_exact(&mut raw_data.latitude);
            let _ = reader.read_exact(&mut raw_data.longitude);
            let _ = reader.read_exact(&mut raw_data.site_height);
            let _ = reader.read_exact(&mut raw_data.feedhorn_height);
            let _ = reader.read_exact(&mut raw_data.calibration_constant);
            let _ = reader.read_exact(&mut raw_data.horizontal_tx_power);
            let _ = reader.read_exact(&mut raw_data.verical_tx_power);
            let _ = reader.read_exact(&mut raw_data.sys_differential_reflectivity);
            let _ = reader.read_exact(&mut raw_data.initial_differential_phase);
            let _ = reader.read_exact(&mut raw_data.vcp_number);

            let vcd = VolumeConstantData::from(raw_data);
            Some(vcd)
        }
    }
}

fn read_elevation_constant_data(data: &[u8], pointer: i32) -> Option<ElevationConstantData> {
    match pointer {
        0 => return None, // no data to read
        _ => {
            let mut reader = std::io::Cursor::new(data);

            // get to the block offset.
            reader.seek(SeekFrom::Start(pointer as u64)).ok()?;
            let mut ecdraw = ElevationConstantDataRaw::default();

            let _ = reader.read_exact(&mut ecdraw.block_type);
            let _ = reader.read_exact(&mut ecdraw.moment_name);
            let _ = reader.read_exact(&mut ecdraw.lrtup);
            let _ = reader.read_exact(&mut ecdraw.atmos_att_factor);
            let _ = reader.read_exact(&mut ecdraw.calibration_constant);

            Some(ElevationConstantData::from(ecdraw))
        }
    }
}

fn read_radial_constant_data(data: &[u8], pointer: i32) -> Option<RadialConstantData> {
    match pointer {
        0 => return None, // no data to read
        _ => {
            let mut reader = std::io::Cursor::new(data);

            // get to the block offset.
            reader.seek(SeekFrom::Start(pointer as u64)).ok()?;
            let mut rcdraw = RadialConstantDataRaw::default();

            let _ = reader.read_exact(&mut rcdraw.block_type);
            let _ = reader.read_exact(&mut rcdraw.moment_name);
            let _ = reader.read_exact(&mut rcdraw.lrtup);
            let _ = reader.read_exact(&mut rcdraw.unambiguous_range);
            let _ = reader.read_exact(&mut rcdraw.noise_level_horiz);
            let _ = reader.read_exact(&mut rcdraw.noise_level_vert);
            let _ = reader.read_exact(&mut rcdraw.nyquist_velocity);

            Some(RadialConstantData::from(rcdraw))
        }
    }
}

fn read_format_moment_block(
    data: &[u8],
    pointer: i32,
) -> Option<DigitalRadarDataGenericFormatBlock> {
    match pointer {
        0 => return None, // no data to read
        _ => {
            let mut reader = std::io::Cursor::new(data);

            // get to the block offset.
            reader.seek(SeekFrom::Start(pointer as u64)).ok()?;
            let mut raw_block = DigitalRadarDataGenericFormatBlockRaw::default();

            let _ = reader.read_exact(&mut raw_block.block_type);
            let _ = reader.read_exact(&mut raw_block.moment_name);
            reader.seek_relative(4); // skip the reserved bytes
            let _ = reader.read_exact(&mut raw_block.num_moment_gates);
            let _ = reader.read_exact(&mut raw_block.num_moment_gates);
            let _ = reader.read_exact(&mut raw_block.data_moment_range);
            let _ = reader.read_exact(&mut raw_block.data_moment_range_sample_interval);
            let _ = reader.read_exact(&mut raw_block.threshold_over);
            let _ = reader.read_exact(&mut raw_block.snr_threshold);
            let _ = reader.read_exact(&mut raw_block.control_flags);
            let _ = reader.read_exact(&mut raw_block.data_word_size);
            let _ = reader.read_exact(&mut raw_block.scale);
            let _ = reader.read_exact(&mut raw_block.offset);
            let _ = reader.read_to_end(&mut raw_block.data);

            let format_block: DigitalRadarDataGenericFormatBlock = raw_block.into();
            Some(format_block)
        }
    }
}
