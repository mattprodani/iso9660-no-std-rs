// SPDX-License-Identifier: (MIT OR Apache-2.0)

use alloc::str;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use nom::bytes::complete::{tag, take};
use nom::combinator::{map, map_res};
use nom::number::complete::*;
use nom::sequence::tuple;
use nom::IResult;
use time::OffsetDateTime;

use super::both_endian::{both_endian16, both_endian32};
use super::date_time::date_time_ascii;
use super::directory_entry::{directory_entry, DirectoryEntryHeader};
use crate::ISOError;

// frankly I think these variants should be holding a struct rather than
// being struct variants. otherwise waste of space.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct PrimaryVolumeDescriptor {
    pub system_identifier: String,
    pub volume_identifier: String,
    pub volume_space_size: u32,
    pub volume_set_size: u16,
    pub volume_sequence_number: u16,
    pub logical_block_size: u16,

    pub path_table_size: u32,
    pub path_table_loc: u32,
    pub optional_path_table_loc: u32,

    pub root_directory_entry: DirectoryEntryHeader,
    pub root_directory_entry_identifier: String,

    pub volume_set_identifier: String,
    pub publisher_identifier: String,
    pub data_preparer_identifier: String,
    pub application_identifier: String,
    pub copyright_file_identifier: String,
    pub abstract_file_identifier: String,
    pub bibliographic_file_identifier: String,

    pub creation_time: OffsetDateTime,
    pub modification_time: OffsetDateTime,
    pub expiration_time: OffsetDateTime,
    pub effective_time: OffsetDateTime,

    pub file_structure_version: u8,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct BootRecordDescriptor {
    pub boot_system_identifier: String,
    pub boot_identifier: String,
    pub data: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct SupplementaryVolumeDescriptor {
    pub type_: u8,
    pub version: u8,
    pub flags: u8,
    pub is_joliet: bool,
}
#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug)]
pub(crate) enum VolumeDescriptor {
    Primary(PrimaryVolumeDescriptor),
    BootRecord(BootRecordDescriptor),
    SupplementaryVolumeDescriptor(SupplementaryVolumeDescriptor),
    VolumeDescriptorSetTerminator,
}

pub fn supplementary_volume_descriptor(input: &[u8]) -> IResult<&[u8], VolumeDescriptor> {
    let (input, type_) = u8(input)?;
    let (input, id) = take(5usize)(input)?;
    let (input, version) = u8(input)?;
    let (input, flags) = u8(input)?;

    // Skip to escape_sequences at offset 88
    // We're at offset 7, need to skip 81 bytes
    let (input, _) = take(81usize)(input)?;

    let (input, escape_seq) = take(32usize)(input)?;

    let mut id_arr = [0u8; 5];
    id_arr.copy_from_slice(id);

    let mut esc_arr = [0u8; 4];
    esc_arr.copy_from_slice(escape_seq);
    let is_joliet = type_ == 2
        && id == b"CD001"
        && version == 1
        && (escape_seq.starts_with(b"%/@")
            || escape_seq.starts_with(b"%/C")
            || escape_seq.starts_with(b"%/E"));

    Ok((
        input,
        VolumeDescriptor::SupplementaryVolumeDescriptor(SupplementaryVolumeDescriptor {
            type_,
            version,
            flags,
            is_joliet,
        }),
    ))
}
//
// pub fn is_joliet(vd: &VolumeDescriptor) -> bool {
//     match vd {
//         VolumeDescriptor::SupplementaryVolumeDescriptor {
//             type_,
//             id,
//             version,
//             flags: _,
//             escape_sequences,
//         } => {
//             *type_ == 2
//                 && id == b"CD001"
//                 && *version == 1
//                 && (escape_sequences.starts_with(b"%/@")
//                     || escape_sequences.starts_with(b"%/C")
//                     || escape_sequences.starts_with(b"%/E"))
//         }
//         _ => false,
//     }
// }
//
impl VolumeDescriptor {
    pub fn parse<E>(bytes: &[u8]) -> Result<Option<VolumeDescriptor>, ISOError<E>> {
        Ok(volume_descriptor(bytes)?.1)
    }
}

fn take_string_trim(count: usize) -> impl Fn(&[u8]) -> IResult<&[u8], String> {
    move |i: &[u8]| {
        map(
            map(map_res(take(count), str::from_utf8), str::trim_end),
            str::to_string,
        )(i)
    }
}

fn boot_record(i: &[u8]) -> IResult<&[u8], VolumeDescriptor> {
    let (i, (boot_system_identifier, boot_identifier, data)) = tuple((
        take_string_trim(32usize),
        take_string_trim(32usize),
        take(1977usize),
    ))(i)?;
    Ok((
        i,
        VolumeDescriptor::BootRecord(BootRecordDescriptor {
            boot_system_identifier,
            boot_identifier,
            data: data.to_vec(),
        }),
    ))
}

fn volume_descriptor(i: &[u8]) -> IResult<&[u8], Option<VolumeDescriptor>> {
    let (i, type_code) = le_u8(i)?;
    let (i, _) = tag("CD001\u{1}")(i)?;
    match type_code {
        0 => map(boot_record, Some)(i),
        1 => map(primary_descriptor, Some)(i),
        2 => map(supplementary_volume_descriptor, Some)(i),
        //3 => map!(volume_partition_descriptor, Some)(i),
        255 => Ok((i, Some(VolumeDescriptor::VolumeDescriptorSetTerminator))),
        _ => Ok((i, None)),
    }
}

fn primary_descriptor(i: &[u8]) -> IResult<&[u8], VolumeDescriptor> {
    let (i, _) = take(1usize)(i)?; // padding
    let (i, system_identifier) = take_string_trim(32usize)(i)?;
    let (i, volume_identifier) = take_string_trim(32usize)(i)?;
    let (i, _) = take(8usize)(i)?; // padding
    let (i, volume_space_size) = both_endian32(i)?;
    let (i, _) = take(32usize)(i)?; // padding
    let (i, volume_set_size) = both_endian16(i)?;
    let (i, volume_sequence_number) = both_endian16(i)?;
    let (i, logical_block_size) = both_endian16(i)?;

    let (i, path_table_size) = both_endian32(i)?;
    let (i, path_table_loc) = le_u32(i)?;
    let (i, optional_path_table_loc) = le_u32(i)?;
    let (i, _) = take(4usize)(i)?; // path_table_loc_be
    let (i, _) = take(4usize)(i)?; // optional_path_table_loc_be

    let (i, root_directory_entry) = directory_entry(i)?;

    let (i, volume_set_identifier) = take_string_trim(128)(i)?;
    let (i, publisher_identifier) = take_string_trim(128)(i)?;
    let (i, data_preparer_identifier) = take_string_trim(128)(i)?;
    let (i, application_identifier) = take_string_trim(128)(i)?;
    let (i, copyright_file_identifier) = take_string_trim(38)(i)?;
    let (i, abstract_file_identifier) = take_string_trim(36)(i)?;
    let (i, bibliographic_file_identifier) = take_string_trim(37)(i)?;

    let (i, creation_time) = date_time_ascii(i)?;
    let (i, modification_time) = date_time_ascii(i)?;
    let (i, expiration_time) = date_time_ascii(i)?;
    let (i, effective_time) = date_time_ascii(i)?;

    let (i, file_structure_version) = le_u8(i)?;

    Ok((
        i,
        VolumeDescriptor::Primary(PrimaryVolumeDescriptor {
            system_identifier,
            volume_identifier,
            volume_space_size,
            volume_set_size,
            volume_sequence_number,
            logical_block_size,

            path_table_size,
            path_table_loc,
            optional_path_table_loc,

            root_directory_entry: root_directory_entry.0,
            root_directory_entry_identifier: root_directory_entry.1,

            volume_set_identifier,
            publisher_identifier,
            data_preparer_identifier,
            application_identifier,
            copyright_file_identifier,
            abstract_file_identifier,
            bibliographic_file_identifier,

            creation_time,
            modification_time,
            expiration_time,
            effective_time,

            file_structure_version,
        }),
    ))
}
