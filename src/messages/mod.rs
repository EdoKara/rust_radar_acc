// EACH WORD IS 4 BYTES; a halfword is 2 bytes.

use std::collections::btree_map::Range;

#[derive(Default, Debug)]
pub struct VolumeHeaderRaw{
    pub volumename:[u8; 12], 
    pub date:[u8; 4], 
    pub time:[u8; 4], 
    pub icao:[u8; 4], 
}

impl VolumeHeaderRaw{
    pub fn new() -> VolumeHeaderRaw{
        VolumeHeaderRaw{
            volumename:[0_u8; 12],
            date: [0_u8; 4],
            time: [0_u8; 4], 
            icao: [0_u8; 4]
        }
    }
}

#[derive(Default, Debug)]
pub struct MessageHeaderRaw{
    pub messagesize:[u8; 2],
    pub rda_redundant_channel: u8,
    pub message_type: u8,
    pub id_seq_no: [u8;2],
    pub julian_date: [u8;2], // julian date - 2440586.5
    pub ms_from_midnight: [u8;4],
    pub n_segments: [u8;2],
    pub message_segment_no: [u8;2],
}

impl MessageHeaderRaw{
    pub fn new() -> MessageHeaderRaw{
        MessageHeaderRaw{
            messagesize:[0_u8; 2],
            rda_redundant_channel: 0_u8,
            message_type: 0_u8,
            id_seq_no: [0_u8;2],
            julian_date: [0_u8;2], // julian date - 2440586.5
            ms_from_midnight: [0_u8;4],
            n_segments: [0_u8;2],
            message_segment_no: [0_u8;2],
        }
    }
}

#[derive(Debug)]
pub struct MessageHeader{
    pub messagesize: i16,
    pub rda_redundant_channel: i8,
    pub message_type: MessageType,
    pub id_seq_no: i16,
    pub julian_date: i16,
    pub ms_from_midnight: i32,
    pub n_segments: i16,
    pub message_segment_no: i16,
}

impl TryFrom<MessageHeaderRaw> for MessageHeader{
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: MessageHeaderRaw) -> Result<Self, Self::Error>{
        Ok(MessageHeader{
        messagesize: i16::from_be_bytes(value.messagesize),
        rda_redundant_channel: value.rda_redundant_channel as i8,
        message_type: collate_message_type(value.message_type as i8).unwrap(),
        id_seq_no: i16::from_be_bytes(value.id_seq_no),
        julian_date: i16::from_be_bytes(value.julian_date),
        ms_from_midnight: i32::from_be_bytes(value.ms_from_midnight),
        n_segments: i16::from_be_bytes(value.n_segments),
        message_segment_no: i16::from_be_bytes(value.message_segment_no)
        })
    }
}

#[derive(Debug)]
pub enum MessageType {
    DigitalRadarData,
    RDAStatusData,
    PerformanceMaintenanceData,
    RDAConsoleMessage,
    RDAVolumeCoveragePattern,
    RDAControlCommand,
    RPGVolumeCoveragePattern,
    ClutterCensorZones,
    RequestForData,
    RPGConsoleMessage,
    LoopBackTestRDA,
    LoopBackTestRPG,
    ClutterFilterBypassMap,
    Spare,
    ClutterFilterMap,
    ReservedFAA,
    RDAAdaptationData,
    Reserved,
    DigitalRadarDataGenericFormat
}

pub fn collate_message_type(message_header_type: i8) -> Result<MessageType, std::io::Error> {

    match message_header_type {
        1 => Ok(MessageType::DigitalRadarData),
        2 => Ok(MessageType::RDAStatusData),
        3 => Ok(MessageType::PerformanceMaintenanceData),
        4 => Ok(MessageType::RDAConsoleMessage),
        5 => Ok(MessageType::RDAVolumeCoveragePattern),
        6 => Ok(MessageType::RDAControlCommand),
        7 => Ok(MessageType::RPGVolumeCoveragePattern),
        8 => Ok(MessageType::ClutterCensorZones),
        9 => Ok(MessageType::RequestForData),
        10 => Ok(MessageType::RPGConsoleMessage),
        11 => Ok(MessageType::LoopBackTestRDA),
        12 => Ok(MessageType::LoopBackTestRPG),
        13 => Ok(MessageType::ClutterFilterBypassMap),
        14 => Ok(MessageType::Spare),
        15 => Ok(MessageType::ClutterFilterMap),
        16 => Ok(MessageType::ReservedFAA),
        17 => Ok(MessageType::ReservedFAA),
        18 => Ok(MessageType::RDAAdaptationData),
        20 => Ok(MessageType::Reserved),
        21 => Ok(MessageType::Reserved),
        22 => Ok(MessageType::Reserved),
        23 => Ok(MessageType::Reserved),
        24 => Ok(MessageType::ReservedFAA),
        25 => Ok(MessageType::ReservedFAA),
        26 => Ok(MessageType::ReservedFAA),
        31 => Ok(MessageType::DigitalRadarDataGenericFormat),
        _ => {Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Number is invalid!"))}
    }

}

#[derive(Debug)]
pub struct VolumeHeader{
    pub volumename: String,
    pub date: i32,
    pub time: i32,
    pub icao: String
}

impl TryFrom<VolumeHeaderRaw> for VolumeHeader {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: VolumeHeaderRaw) -> Result<Self, Self::Error> {
        Ok(VolumeHeader{
            volumename: std::str::from_utf8(&value.volumename)?.to_string(),
            date: i32::from_be_bytes(value.date),
            time: i32::from_be_bytes(value.time),
            icao: std::str::from_utf8(&value.icao)?.to_string()
        }
        )
    }
}

pub struct RawClutterFilterMapMetadata {
    pub map_generation_date: [u8;2],
    pub map_generation_time: [u8;2],
    pub num_elevation_segments: [u8;2],
    pub elevation_segments: Vec<ElevationSegment>
}

pub struct ClutterFilterMapMetadata{
    pub map_generation_date: i16,
    pub map_generation_time: i16,
    pub num_elevation_segments: i16,
    pub elevation_segments: Vec<ElevationSegment>
}

pub struct ElevationSegment { 
    pub azimuth_segments: Vec<AzimuthSegment>
}

pub struct RawAzimuthSegment {
    pub num_rangezones: [u8; 2],
    pub range_zones: Vec<RangeZone>
}

pub struct AzimuthSegment {
    pub num_rangezones: i16, 
    pub range_zones: Vec<RangeZone>
}

pub struct RawRangeZone{ 
    pub range_zone_num: i16,
    pub opcode: [u8;2],
    pub endrange: [u8;2]
}

impl RawRangeZone{
    pub fn new() -> RawRangeZone{
        RawRangeZone{
            range_zone_num: 0,
            opcode: [0_u8; 2],
            endrange: [0_u8;2]
        }
    }
}

impl TryFrom<RawRangeZone> for RangeZone{
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: RawRangeZone) -> Result<Self, Self::Error>{
        Ok(RangeZone{
            range_zone_num: value.range_zone_num,
            opcode: i16::from_be_bytes(value.opcode),
            endrange: i16::from_be_bytes(value.endrange)
        })
    }
}


pub struct RangeZone{
    pub range_zone_num: i16,
    pub opcode: i16,
    pub endrange: i16
}

impl RangeZone{
    pub fn new() -> RangeZone{
        RangeZone{
            range_zone_num: 0,
            opcode: 0,
            endrange: 0 
        }
    }
}