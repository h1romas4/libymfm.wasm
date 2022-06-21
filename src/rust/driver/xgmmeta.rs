// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
use nom::bytes::complete::{tag, take};
use nom::number::complete::{le_u16, le_u32, le_u8};
use nom::IResult;

use crate::driver::meta::Jsonlize;
use crate::driver::gd3meta::{parse_gd3, Gd3};

///
/// VDP Mode
///
#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum VDPMode {
    NTSC = 60,
    PAL = 50,
}

pub const XGM_SAMPLE_DATA_BLOC_ADDRESS: usize = 0x104;

///
/// https://github.com/Stephane-D/SGDK/blob/master/bin/xgm.txt
///
#[derive(Deserialize, Serialize, Debug)]
pub struct XgmHeader {
    pub sample_id_table: Vec<(u16, u16)>,
    pub sample_data_bloc_size: u16,
    pub version: u8,
    pub vdp_mode: VDPMode,
    pub gd3_tag: bool,
    pub multi_track_file: bool,
    pub music_data_bloc_size: u32,
}

///
/// Parse XGM header
///
fn parse_xgm_header(i: &[u8]) -> IResult<&[u8], XgmHeader> {
    let (i, _) = tag("XGM ")(i)?;
    let (i, sample_id_table_block) = take(252usize)(i)?;
    let (i, sample_data_bloc_size) = le_u16(i)?;
    let (i, version) = le_u8(i)?;
    let (i, flags) = le_u8(i)?;
    let vdp_mode = if flags & 0b00000001 == 0 { VDPMode::NTSC } else { VDPMode::PAL };
    let gd3_tag = flags & 0b00000010 != 0;
    let multi_track_file = flags & 0b00000100 != 0;
    let (i, /* sample data bloc */ _) = take(sample_data_bloc_size as u32 * 256)(i)?;
    let (i, music_data_bloc_size) = le_u32(i)?;

    // extract sampling id table
    let mut sample_id_table = Vec::new();
    for index in 0..62 {
        let i = index * 4;
        let address = u16::from_le_bytes(
            sample_id_table_block[i..(i + 2)]
                .try_into()
                .unwrap(),
        );
        // An empty entry should have its address set to $FFFF and size set to $0001.
        // There are cases where the size is 0, so it is not handled.
        if address == 0xffff {
            continue;
        }
        let size = u16::from_le_bytes(
            sample_id_table_block[(i + 2)..(i + 4)]
                .try_into()
                .unwrap(),
        );
        sample_id_table.insert(index, (address, size));
    }

    Ok((
        i,
        XgmHeader {
            sample_id_table,
            sample_data_bloc_size,
            version,
            vdp_mode,
            gd3_tag,
            multi_track_file,
            music_data_bloc_size,
        },
    ))
}

///
/// Parse XGM meta
///
pub(crate) fn parse_xgm_meta(xgmdata: &[u8]) -> Result<(XgmHeader, Gd3), &'static str> {
    let header = match parse_xgm_header(xgmdata) {
        Ok((_, header)) => header,
        Err(_) => return Err("xgm header parse error."),
    };
    let gd3 = if header.gd3_tag {
        match parse_gd3(
            &xgmdata[(0x108 + (header.sample_data_bloc_size as u32 * 256) + header.music_data_bloc_size)
                as usize..],
        ) {
            Ok((_, gd3)) => gd3,
            Err(_) => Gd3::default(), // blank values
        }
    } else {
        Gd3::default() // blank values
    };

    Ok((header, gd3))
}

impl Jsonlize for XgmHeader {}

#[cfg(test)]
mod tests {
    use super::parse_xgm_meta;
    use super::Jsonlize;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_1() {
        parse("./docs/vgm/sor2.xgm")
    }

    #[test]
    fn test_2() {
        parse("./docs/vgm/sor3.xgm")
    }

    fn parse(filepath: &str) {
        // load sn76489 vgm file
        let mut file = File::open(filepath).unwrap();
        let mut buffer = Vec::new();
        let _ = file.read_to_end(&mut buffer).unwrap();
        let (header, gd3) = parse_xgm_meta(&buffer).unwrap();

        println!("{:#?}", header);
        println!("{:#?}", gd3);
        println!("{:#?}", header.get_json());
        println!("{:#?}", gd3.get_json());
    }
}
