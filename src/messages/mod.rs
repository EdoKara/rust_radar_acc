use packed_struct::prelude::*;
// EACH WORD IS 4 BYTES; a halfword is 2 bytes.

pub const HALFWORD_SIZE: usize = 2;

use std::{collections::btree_map::Range, io::Error};

use crate::MESSAGE_RECORD_SIZE;

#[derive(Default, Debug)]
pub struct VolumeHeaderRaw {
    pub volumename: [u8; 12],
    pub date: [u8; 4],
    pub time: [u8; 4],
    pub icao: [u8; 4],
}

impl VolumeHeaderRaw {
    pub fn new() -> VolumeHeaderRaw {
        VolumeHeaderRaw {
            volumename: [0_u8; 12],
            date: [0_u8; 4],
            time: [0_u8; 4],
            icao: [0_u8; 4],
        }
    }
}

#[derive(Default, Debug)]
pub struct MessageHeaderRaw {
    pub messagesize: [u8; 2],
    pub rda_redundant_channel: [u8; 1],
    pub message_type: [u8; 1],
    pub id_seq_no: [u8; 2],
    pub julian_date: [u8; 2], // julian date - 2440586.5
    pub ms_from_midnight: [u8; 4],
    pub n_segments: [u8; 2],
    pub message_segment_no: [u8; 2],
}

pub const MESSAGE_HEADER_SIZE: usize = 16;
impl MessageHeaderRaw {
    pub fn new() -> MessageHeaderRaw {
        MessageHeaderRaw {
            messagesize: [0_u8; 2],
            rda_redundant_channel: [0_u8],
            message_type: [0_u8],
            id_seq_no: [0_u8; 2],
            julian_date: [0_u8; 2], // julian date - 2440586.5
            ms_from_midnight: [0_u8; 4],
            n_segments: [0_u8; 2],
            message_segment_no: [0_u8; 2],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageHeader {
    pub messagesize: i16,
    pub rda_redundant_channel: i8,
    pub message_type: MessageType,
    pub id_seq_no: i16,
    pub julian_date: i16,
    pub ms_from_midnight: i32,
    pub n_segments: i16,
    pub message_segment_no: i16,
}

impl TryFrom<MessageHeaderRaw> for MessageHeader {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: MessageHeaderRaw) -> Result<Self, Self::Error> {
        Ok(MessageHeader {
            messagesize: i16::from_be_bytes(value.messagesize),
            rda_redundant_channel: i8::from_be_bytes(value.rda_redundant_channel),
            message_type: collate_message_type(i8::from_be_bytes(value.message_type))
                .map_err(|e| println!("failed to parse message type.\n error:{e}"))
                .unwrap(),
            id_seq_no: i16::from_be_bytes(value.id_seq_no),
            julian_date: i16::from_be_bytes(value.julian_date),
            ms_from_midnight: i32::from_be_bytes(value.ms_from_midnight),
            n_segments: i16::from_be_bytes(value.n_segments),
            message_segment_no: i16::from_be_bytes(value.message_segment_no),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    DigitalRadarDataGenericFormat,
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
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Number is invalid!",
        )),
    }
}

#[derive(Debug)]
pub struct VolumeHeader {
    pub volumename: String,
    pub date: i32,
    pub time: i32,
    pub icao: String,
}

impl TryFrom<VolumeHeaderRaw> for VolumeHeader {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: VolumeHeaderRaw) -> Result<Self, Self::Error> {
        Ok(VolumeHeader {
            volumename: std::str::from_utf8(&value.volumename)?.to_string(),
            date: i32::from_be_bytes(value.date),
            time: i32::from_be_bytes(value.time),
            icao: std::str::from_utf8(&value.icao)?.to_string(),
        })
    }
}

#[derive(Debug)]
pub struct RawClutterFilterMapMetadata {
    pub map_generation_date: [u8; 2],
    pub map_generation_time: [u8; 2],
    pub num_elevation_segments: [u8; 2],
    pub elevation_segments: Vec<RawElevationSegment>,
}

impl RawClutterFilterMapMetadata {
    pub fn new() -> RawClutterFilterMapMetadata {
        RawClutterFilterMapMetadata {
            map_generation_date: [0_u8; 2],
            map_generation_time: [0_u8; 2],
            num_elevation_segments: [0_u8; 2],
            elevation_segments: vec![RawElevationSegment::new(); 5],
        }
    }
}

pub struct ClutterFilterMapMetadata {
    pub map_generation_date: i16,
    pub map_generation_time: i16,
    pub num_elevation_segments: i16,
    pub elevation_segments: Vec<ElevationSegment>,
}

pub struct ElevationSegment {
    pub azimuth_segments: Vec<AzimuthSegment>,
}

impl TryFrom<RawElevationSegment> for ElevationSegment {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: RawElevationSegment) -> Result<Self, Self::Error> {
        //let mut convs: Vec<Result<AzimuthSegment, Box<dyn std::error::Error>>> = Vec::new();
        let mut convs: Vec<AzimuthSegment> = Vec::new();
        for aseg in value.azimuth_segments.iter() {
            convs.push(AzimuthSegment::try_from(aseg.clone()).unwrap());
        }
        Ok(ElevationSegment {
            azimuth_segments: convs,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RawElevationSegment {
    pub azimuth_segments: Vec<RawAzimuthSegment>,
}

impl RawElevationSegment {
    pub fn new() -> RawElevationSegment {
        RawElevationSegment {
            azimuth_segments: vec![RawAzimuthSegment::new(); 360],
        }
    }
}

#[derive(Debug, Clone)]
pub struct RawAzimuthSegment {
    pub num_rangezones: [u8; 2],
    pub range_zones: Vec<RangeZone>,
}

impl RawAzimuthSegment {
    pub fn new() -> RawAzimuthSegment {
        RawAzimuthSegment {
            num_rangezones: [0; 2],
            range_zones: Vec::new(),
        }
    }
}

impl TryFrom<RawAzimuthSegment> for AzimuthSegment {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: RawAzimuthSegment) -> Result<Self, Self::Error> {
        Ok(AzimuthSegment {
            num_rangezones: i16::from_be_bytes(value.num_rangezones),
            range_zones: value.range_zones,
        })
    }
}

pub struct AzimuthSegment {
    pub num_rangezones: i16,
    pub range_zones: Vec<RangeZone>,
}

impl AzimuthSegment {
    pub fn new(&self, nrangezones: Option<i16>) -> AzimuthSegment {
        match nrangezones {
            Some(num_rangezone) => AzimuthSegment {
                num_rangezones: num_rangezone,
                range_zones: vec![RangeZone::new(); self.num_rangezones as usize],
            },
            None => AzimuthSegment {
                num_rangezones: 20,
                range_zones: vec![RangeZone::new(); self.num_rangezones as usize],
            },
        }
    }
}

pub struct RawRangeZone {
    pub range_zone_num: i16,
    pub opcode: [u8; 2],
    pub endrange: [u8; 2],
}

impl RawRangeZone {
    pub fn new() -> RawRangeZone {
        RawRangeZone {
            range_zone_num: 0,
            opcode: [0_u8; 2],
            endrange: [0_u8; 2],
        }
    }
}

impl TryFrom<RawRangeZone> for RangeZone {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: RawRangeZone) -> Result<Self, Self::Error> {
        Ok(RangeZone {
            range_zone_num: value.range_zone_num,
            opcode: i16::from_be_bytes(value.opcode),
            endrange: i16::from_be_bytes(value.endrange),
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RangeZone {
    pub range_zone_num: i16,
    pub opcode: i16,
    pub endrange: i16,
}

impl RangeZone {
    pub fn new() -> RangeZone {
        RangeZone {
            range_zone_num: 0,
            opcode: 0,
            endrange: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct DigitalRadarDataGenericFormatHeader {
    pub radar_identifier: String,
    pub collection_time: i32,
    pub modified_julian_date: i16,
    pub azimuth_number: i16,
    pub azimuth_angle: f32,
    pub compression_indicator: u8,
    pub spare_byte: u8,
    pub radial_length: i16,
    pub az_res_spacing: u8,
    pub radial_status: u8,
    pub elevation_number: i8,
    pub cut_sector_number: i8,
    pub elevation_angle: f32,
    pub radial_spot_blanking_status: u8,
    pub azimuth_indexing_mode: u8,
    pub data_block_count: i16,
    pub const_vol_data_block_pointer: i32,
    pub const_elevation_data_block_pointer: i32,
    pub const_radial_data_block_pointer: i32,
    pub reflectivity_block_pointer: i32,
    pub velocity_block_pointer: i32,
    pub sw_block_pointer: i32,
    pub diff_ref_block_pointer: i32,
    pub phi_block_pointer: i32,
    pub rho_block_pointer: i32,
}

pub const DIGITAL_RADAR_DATA_GENERIC_FORMAT_HEADER_SIZE: usize = 68;
#[derive(Debug)]
pub struct DigitalRadarDataGenericFormatHeaderRaw {
    pub radar_identifier: [u8; 4],
    pub collection_time: [u8; 4],
    pub modified_julian_date: [u8; 2],
    pub azimuth_number: [u8; 2],
    pub azimuth_angle: [u8; 4],
    pub compression_indicator: [u8; 1],
    pub spare_byte: [u8; 1],
    pub radial_length: [u8; 2],
    pub az_res_spacing: [u8; 1],
    pub radial_status: [u8; 1],
    pub elevation_number: [u8; 1],
    pub cut_sector_number: [u8; 1],
    pub elevation_angle: [u8; 4],
    pub radial_spot_blanking_status: [u8; 1],
    pub azimuth_indexing_mode: [u8; 1],
    pub data_block_count: [u8; 2],
    pub const_vol_data_block_pointer: [u8; 4],
    pub const_elevation_data_block_pointer: [u8; 4],
    pub const_radial_data_block_pointer: [u8; 4],
    pub reflectivity_block_pointer: [u8; 4],
    pub velocity_block_pointer: [u8; 4],
    pub sw_block_pointer: [u8; 4],
    pub diff_ref_block_pointer: [u8; 4],
    pub phi_block_pointer: [u8; 4],
    pub rho_block_pointer: [u8; 4],
}

impl From<DigitalRadarDataGenericFormatHeaderRaw> for DigitalRadarDataGenericFormatHeader {
    fn from(value: DigitalRadarDataGenericFormatHeaderRaw) -> Self {
        DigitalRadarDataGenericFormatHeader {
            radar_identifier: std::str::from_utf8(&value.radar_identifier)
                .unwrap()
                .to_string(),
            collection_time: i32::from_be_bytes(value.collection_time),
            modified_julian_date: i16::from_be_bytes(value.modified_julian_date),
            azimuth_number: i16::from_be_bytes(value.azimuth_number),
            azimuth_angle: f32::from_be_bytes(value.azimuth_angle),
            compression_indicator: value.compression_indicator[0],
            spare_byte: value.spare_byte[0],
            radial_length: i16::from_be_bytes(value.radial_length),
            az_res_spacing: value.az_res_spacing[0],
            radial_status: value.radial_status[0],
            elevation_number: value.elevation_number[0] as i8,
            cut_sector_number: value.cut_sector_number[0] as i8,
            elevation_angle: f32::from_be_bytes(value.elevation_angle),
            radial_spot_blanking_status: value.radial_spot_blanking_status[0],
            azimuth_indexing_mode: value.azimuth_indexing_mode[0],
            data_block_count: i16::from_be_bytes(value.data_block_count),
            const_vol_data_block_pointer: i32::from_be_bytes(value.const_vol_data_block_pointer),
            const_elevation_data_block_pointer: i32::from_be_bytes(
                value.const_elevation_data_block_pointer,
            ),
            const_radial_data_block_pointer: i32::from_be_bytes(
                value.const_radial_data_block_pointer,
            ),
            reflectivity_block_pointer: i32::from_be_bytes(value.reflectivity_block_pointer),
            velocity_block_pointer: i32::from_be_bytes(value.velocity_block_pointer),
            sw_block_pointer: i32::from_be_bytes(value.sw_block_pointer),
            diff_ref_block_pointer: i32::from_be_bytes(value.diff_ref_block_pointer),
            phi_block_pointer: i32::from_be_bytes(value.phi_block_pointer),
            rho_block_pointer: i32::from_be_bytes(value.rho_block_pointer),
        }
    }
}

impl DigitalRadarDataGenericFormatHeaderRaw {
    pub fn new() -> DigitalRadarDataGenericFormatHeaderRaw {
        DigitalRadarDataGenericFormatHeaderRaw {
            radar_identifier: [0_u8; 4],
            collection_time: [0_u8; 4],
            modified_julian_date: [0_u8; 2],
            azimuth_number: [0_u8; 2],
            azimuth_angle: [0_u8; 4],
            compression_indicator: [0_u8],
            spare_byte: [0_u8],
            radial_length: [0_u8; 2],
            az_res_spacing: [0_u8],
            radial_status: [0_u8; 1],
            elevation_number: [0_u8; 1],
            cut_sector_number: [0_u8; 1],
            elevation_angle: [0_u8; 4],
            radial_spot_blanking_status: [0_u8; 1],
            azimuth_indexing_mode: [0_u8; 1],
            data_block_count: [0_u8; 2],
            const_vol_data_block_pointer: [0_u8; 4],
            const_elevation_data_block_pointer: [0_u8; 4],
            const_radial_data_block_pointer: [0_u8; 4],
            reflectivity_block_pointer: [0_u8; 4],
            velocity_block_pointer: [0_u8; 4],
            sw_block_pointer: [0_u8; 4],
            diff_ref_block_pointer: [0_u8; 4],
            phi_block_pointer: [0_u8; 4],
            rho_block_pointer: [0_u8; 4],
        }
    }
}

impl Default for DigitalRadarDataGenericFormatHeaderRaw {
    fn default() -> Self {
        DigitalRadarDataGenericFormatHeaderRaw::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressedRecord {
    pub record_type: RecordType,
    pub buf: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordType {
    MetadataRecord,
    NormalRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataMessageBlock {
    pub blocktype: MetadataBlockType,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataBlockType {
    Empty,
    Populated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataMessage {
    pub header: MessageHeader,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegularMessageBlock {
    pub header: MessageHeader,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DigitalRadarDataGenericFormat {
    pub header: DigitalRadarDataGenericFormatHeader,
    pub data: Vec<u8>,
    pub volume_constant_data: Option<VolumeConstantData>,
    pub elevation_constant_data: Option<ElevationConstantData>,
    pub radial_constant_data: Option<RadialConstantData>,
    pub moment_blocks: Vec<Option<DigitalRadarDataGenericFormatBlock>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RadialConstantData {
    pub block_type: BlockType,
    pub moment_name: DataMoment,
    pub lrtup: u16,
    pub unambiguous_range: f32, // scaled SInteger*2
    pub noise_level_horiz: f32,
    pub noise_level_vert: f32,
    pub nyquist_velocity: f32, // scaled SInteger*2
    spare: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RadialConstantDataRaw {
    pub block_type: [u8; 1],
    pub moment_name: [u8; 3],
    pub lrtup: [u8; 2],
    pub unambiguous_range: [u8; 2], // scaled SInteger*2
    pub noise_level_horiz: [u8; 4],
    pub noise_level_vert: [u8; 4],
    pub nyquist_velocity: [u8; 2], // scaled SInteger*2
    spare: Vec<u8>,
}

impl From<RadialConstantDataRaw> for RadialConstantData {
    fn from(value: RadialConstantDataRaw) -> Self {
        let btype: BlockType = match str::from_utf8(&value.block_type).unwrap() {
            "D" => BlockType::D,
            "R" => BlockType::R,
            _ => panic!("Unknown Block type!"),
        };

        let moment_name: DataMoment = match str::from_utf8(&value.moment_name).unwrap() {
            "REF" => DataMoment::Reflectivity,
            "VEL" => DataMoment::Velocity,
            "SW " => DataMoment::SpectrumWidth,
            "ZDR" => DataMoment::DifferentialReflectivity,
            "RHO" => DataMoment::CorrelationCoefficient,
            "PHI" => DataMoment::DifferentialPhase,
            "VOL" => DataMoment::Volume,
            "ELV" => DataMoment::Elevation,
            "RAD" => DataMoment::Radial,
            "CFP" => DataMoment::ClutterFilterPowerRemoved,
            _ => panic!("Unknown moment name! {:x?}", &value.moment_name),
        };

        println!("Block type: {:x?}", &value.moment_name);
        println!(
            "Block type: {:x?}",
            str::from_utf8(&value.moment_name).unwrap()
        );

        RadialConstantData {
            block_type: btype,
            moment_name,
            lrtup: u16::from_be_bytes(value.lrtup),
            unambiguous_range: (i16::from_be_bytes(value.unambiguous_range) as f32)
                / (10_i16.pow(1) as f32),
            noise_level_horiz: f32::from_be_bytes(value.noise_level_horiz),
            noise_level_vert: f32::from_be_bytes(value.noise_level_vert),
            nyquist_velocity: (i16::from_be_bytes(value.nyquist_velocity) as f32)
                / (10_i16.pow(2) as f32),
            spare: value.spare,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElevationConstantData {
    pub block_type: BlockType,
    pub moment_name: DataMoment,
    pub lrtup: u16,
    pub atmos_att_factor: f32, // scaled SInteger*2
    pub calibration_constant: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ElevationConstantDataRaw {
    pub block_type: [u8; 1],
    pub moment_name: [u8; 3],
    pub lrtup: [u8; 2],
    pub atmos_att_factor: [u8; 2], // scaled SInteger*2
    pub calibration_constant: [u8; 4],
}

impl From<ElevationConstantDataRaw> for ElevationConstantData {
    fn from(value: ElevationConstantDataRaw) -> ElevationConstantData {
        let btype: BlockType = match str::from_utf8(&value.block_type).unwrap() {
            "D" => BlockType::D,
            "R" => BlockType::R,
            _ => panic!("Unknown Block type!"),
        };

        println!("Block type: {:x?}", &value.moment_name);
        println!(
            "Block type: {:x?}",
            str::from_utf8(&value.moment_name).unwrap()
        );

        let moment_name: DataMoment = match str::from_utf8(&value.moment_name).unwrap() {
            "REF" => DataMoment::Reflectivity,
            "VEL" => DataMoment::Velocity,
            "SW " => DataMoment::SpectrumWidth,
            "ZDR" => DataMoment::DifferentialReflectivity,
            "RHO" => DataMoment::CorrelationCoefficient,
            "PHI" => DataMoment::DifferentialPhase,
            "VOL" => DataMoment::Volume,
            "ELV" => DataMoment::Elevation,
            "RAD" => DataMoment::Radial,
            "CFP" => DataMoment::ClutterFilterPowerRemoved,
            _ => panic!("Unknown moment name! {:x?}", &value.moment_name),
        };

        ElevationConstantData {
            block_type: btype,
            moment_name,
            lrtup: u16::from_be_bytes(value.lrtup),
            atmos_att_factor: (i16::from_be_bytes(value.atmos_att_factor) as f32)
                / (10_i16.pow(3) as f32),
            calibration_constant: f32::from_be_bytes(value.calibration_constant),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VolumeConstantData {
    pub block_type: BlockType,
    pub moment_name: DataMoment,
    pub lrtup: u16,
    pub version_number: u8,
    pub version_Number_minor: u8,
    pub latitude: f32,
    pub longitude: f32,
    pub site_height: f32, // SInteger*2
    pub feedhorn_height: i16,
    pub calibration_constant: f32,
    pub horizontal_tx_power: f32,
    pub verical_tx_power: f32,
    pub sys_differential_reflectivity: f32,
    pub initial_differential_phase: f32,
    pub vcp_number: u16,
    spare: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VolumeConstantDataRaw {
    pub block_type: [u8; 1],
    pub moment_name: [u8; 3],
    pub lrtup: [u8; 2],
    pub version_number: [u8; 1],
    pub version_Number_minor: [u8; 1],
    pub latitude: [u8; 4],
    pub longitude: [u8; 4],
    pub site_height: [u8; 2], // SInteger*2
    pub feedhorn_height: [u8; 2],
    pub calibration_constant: [u8; 4],
    pub horizontal_tx_power: [u8; 4],
    pub verical_tx_power: [u8; 4],
    pub sys_differential_reflectivity: [u8; 4],
    pub initial_differential_phase: [u8; 4],
    pub vcp_number: [u8; 2],
    spare: [u8; 2],
}

impl From<VolumeConstantDataRaw> for VolumeConstantData {
    fn from(value: VolumeConstantDataRaw) -> VolumeConstantData {
        let btype: BlockType = match str::from_utf8(&value.block_type).unwrap() {
            "D" => BlockType::D,
            "R" => BlockType::R,
            _ => panic!("Unknown Block type!"),
        };

        println!("Block type: {:x?}", &value.moment_name);
        println!(
            "Block type: {:x?}",
            str::from_utf8(&value.moment_name).unwrap()
        );

        let moment_name: DataMoment = match str::from_utf8(&value.moment_name).unwrap() {
            "REF" => DataMoment::Reflectivity,
            "VEL" => DataMoment::Velocity,
            "SW " => DataMoment::SpectrumWidth,
            "ZDR" => DataMoment::DifferentialReflectivity,
            "RHO" => DataMoment::CorrelationCoefficient,
            "PHI" => DataMoment::DifferentialPhase,
            "VOL" => DataMoment::Volume,
            "ELV" => DataMoment::Elevation,
            "RAD" => DataMoment::Radial,
            "CFP" => DataMoment::ClutterFilterPowerRemoved,
            _ => panic!("Unknown moment name! {:x?}", &value.moment_name),
        };

        VolumeConstantData {
            block_type: btype,
            moment_name,
            lrtup: u16::from_be_bytes(value.lrtup),
            version_number: value.version_number[0],
            version_Number_minor: value.version_Number_minor[0],
            latitude: f32::from_be_bytes(value.latitude),
            longitude: f32::from_be_bytes(value.longitude),
            site_height: (i16::from_be_bytes(value.site_height) as f32) / (10_i16.pow(0) as f32),
            feedhorn_height: i16::from_be_bytes(value.feedhorn_height),
            calibration_constant: f32::from_be_bytes(value.calibration_constant),
            horizontal_tx_power: f32::from_be_bytes(value.horizontal_tx_power),
            verical_tx_power: f32::from_be_bytes(value.verical_tx_power),
            sys_differential_reflectivity: f32::from_be_bytes(value.sys_differential_reflectivity),
            initial_differential_phase: f32::from_be_bytes(value.initial_differential_phase),
            vcp_number: u16::from_be_bytes(value.vcp_number),
            spare: value.spare.to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DigitalRadarDataGenericFormatBlock {
    pub block_type: BlockType,
    pub moment_name: DataMoment,
    reserved: Vec<u8>,
    pub num_moment_gates: i16,
    pub data_moment_range: f32, // this field is weird, it's a scaled int*2 with a float with three decimal places.
    pub data_moment_range_sample_interval: f32, // this is also a scaled int*2 with three decimal places.
    pub threshold_over: f32,                    // this is a scaled int*2 with one decimal place.
    pub snr_threshold: f32, // this is a scaled signed int*2 as a float with one decimal place.
    pub control_flags: u8,  // this is a bitfield.
    pub data_word_size: u8, // this is the size of the data word in bytes.
    pub scale: f32,
    pub offset: f32,
    pub data: Vec<u8>, // this is the actual data for the block.
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DigitalRadarDataGenericFormatBlockRaw {
    pub block_type: [u8; 1],
    pub moment_name: [u8; 3],
    reserved: [u8; 4],
    pub num_moment_gates: [u8; 2],
    pub data_moment_range: [u8; 2], // this field is weird, it's a scaled int*2 with a float with three decimal places.
    pub data_moment_range_sample_interval: [u8; 2], // this is also a scaled int*2 with three decimal places.
    pub threshold_over: [u8; 2], // this is a scaled int*2 with one decimal place.
    pub snr_threshold: [u8; 2],  // this is a scaled signed int*2 as a float with one decimal place.
    pub control_flags: [u8; 1],  // this is a bitfield.
    pub data_word_size: [u8; 1], // this is the size of the data word in bytes.
    pub scale: [u8; 4],
    pub offset: [u8; 4],
    pub data: Vec<u8>, // this is the actual data for the block.
}

impl From<DigitalRadarDataGenericFormatBlockRaw> for DigitalRadarDataGenericFormatBlock {
    fn from(value: DigitalRadarDataGenericFormatBlockRaw) -> DigitalRadarDataGenericFormatBlock {
        let btype: BlockType = match str::from_utf8(&value.block_type).unwrap() {
            "D" => BlockType::D,
            "R" => BlockType::R,
            _ => panic!("Unknown Block type!"),
        };

        println!("Block type: {:x?}", &value.moment_name);
        println!(
            "Block type: {:x?}",
            str::from_utf8(&value.moment_name).unwrap()
        );

        let moment_name: DataMoment = match str::from_utf8(&value.moment_name).unwrap() {
            "REF" => DataMoment::Reflectivity,
            "VEL" => DataMoment::Velocity,
            "SW " => DataMoment::SpectrumWidth,
            "ZDR" => DataMoment::DifferentialReflectivity,
            "RHO" => DataMoment::CorrelationCoefficient,
            "PHI" => DataMoment::DifferentialPhase,
            "VOL" => DataMoment::Volume,
            "ELV" => DataMoment::Elevation,
            "RAD" => DataMoment::Radial,
            "CFP" => DataMoment::ClutterFilterPowerRemoved,
            _ => panic!("Unknown moment name! {:x?}", &value.moment_name),
        };

        DigitalRadarDataGenericFormatBlock {
            block_type: btype,
            moment_name,
            reserved: value.reserved.to_vec(),
            num_moment_gates: i16::from_be_bytes(value.num_moment_gates),
            data_moment_range: (i16::from_be_bytes(value.data_moment_range) as f32)
                / (10_i16.pow(3) as f32),
            data_moment_range_sample_interval: (i16::from_be_bytes(
                value.data_moment_range_sample_interval,
            ) as f32)
                / (10_i16.pow(3) as f32),
            threshold_over: (i16::from_be_bytes(value.threshold_over) as f32)
                / (10_i16.pow(1) as f32),
            snr_threshold: (i16::from_be_bytes(value.snr_threshold) as f32)
                / (10_i16.pow(1) as f32),
            control_flags: value.control_flags[0],
            data_word_size: value.data_word_size[0],
            scale: f32::from_be_bytes(value.scale),
            offset: f32::from_be_bytes(value.offset),
            data: value.data,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum BlockType {
    R,
    #[default]
    D,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DataMoment {
    #[default]
    Reflectivity,
    Velocity,
    SpectrumWidth,
    DifferentialReflectivity,
    CorrelationCoefficient,
    DifferentialPhase,
    Volume,
    Elevation,
    Radial,
    ClutterFilterPowerRemoved,
}
