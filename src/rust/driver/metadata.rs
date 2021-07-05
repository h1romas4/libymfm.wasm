use nom::number::complete::{le_u8, le_u16, le_u32};
use nom::bytes::complete::{tag, take};
use nom::IResult;

///
/// https://vgmrips.net/wiki/VGM_Specification
///
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct VgmHeader {
    pub eof: u32,
    pub version: u32,
    pub clock_sn76489: u32,
    pub clock_ym2413: u32,
    pub offset_gd3: u32,
    pub total_samples: u32,
    pub offset_loop: u32,
    pub loop_samples: u32,
    pub rate: u32,
    pub sn76489_fb: u16,
    pub sn76489_w: u8,
    pub sn76489_f: u8,
    pub clock_ym2612: u32,
    pub clock_ym2151: u32,
    pub vgm_data_offset: u32,
    pub sega_pcm_clock: u32,
    pub spcm_interface: u32,
    pub clock_rf5c68: u32,
    pub clock_ym2203: u32,
    pub clock_ym2608: u32,
    pub clock_ym2610_b: u32,
    pub clock_ym3812: u32,
    pub clock_ym3526: u32,
    pub clock_y8950: u32,
    pub clock_ymf262: u32,
    pub clock_ymf278_b: u32,
    pub clock_ym271: u32,
    pub clock_ymz280b: u32,
    pub clock_rf5c164: u32,
    pub clock_pwm: u32,
    pub clock_ay8910: u32,
    pub ay8910_chip_type: u8,
    pub ay8910_flag: u16,
    pub volume_modifier: u8,
    pub reserved01: u8,
    pub loop_base: u8,
    pub loop_modifier: u8,
    pub clock_gb_dmg: u32,
    pub clock_nes_apu: u32,
    pub clock_multi_pcm: u32,
    pub clock_upd7759: u32,
    pub clock_okim6258: u32,
    pub okmi6258_flag: u8,
    pub k054539_flag: u8,
    pub c140_chip_type : u8,
    pub reserved02: u8,
    pub clock_okim6295: u32,
    pub clock_k051649: u32,
    pub clock_k054539: u32,
    pub clock_huc6280: u32,
    pub clock_c140: u32,
    pub clock_k053260: u32,
    pub clock_pokey: u32,
    pub clock_qsound: u32,
    pub clock_scsp: u32,
    pub extra_hdr_ofs: u32,
    pub clock_wonder_swan: u32,
    pub clock_vsu: u32,
    pub clock_saa1099: u32,
    pub clock_es5503: u32,
    pub clock_es5506: u32,
    pub es5503_amount_channel: u8,
    pub es5506_amount_channel: u8,
    pub c352_clock_divider: u8,
    pub reserved03: u8,
    pub clock_x1_010: u32,
    pub clock_c352: u32,
    pub clock_ga20: u32,
    pub reserved04: u32,
    pub reserved05: u32,
    pub reserved06: u32,
    pub reserved07: u32,
    pub reserved08: u32,
    pub reserved09: u32,
    pub reserved10: u32
}

///
/// https://vgmrips.net/wiki/GD3_Specification
///
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Gd3 {
    pub track_name: String,
    pub track_name_j: String,
    pub game_name: String,
    pub game_name_j: String,
    pub system_name: String,
    pub system_name_j: String,
    pub track_author: String,
    pub track_author_j: String,
    pub date: String,
    pub converted: String
}

///
/// parse_vgm_header
///
fn parse_vgm_header(i: &[u8]) -> IResult<&[u8], VgmHeader> {
    let (i, _) = tag("Vgm ")(i)?;
    let (i, eof) = le_u32(i)?;
    let (i, version) = take(4usize)(i)?;
    let version = version.iter().rev().map(|n| format!("{:02X}", n)).collect::<String>();
    let version = version.parse().unwrap_or(0);
    let (i, clock_sn76489) = le_u32(i)?;
    let (i, clock_ym2413) = le_u32(i)?;
    let (i, offset_gd3) = le_u32(i)?;
    let (i, total_samples) = le_u32(i)?;
    let (i, offset_loop) = le_u32(i)?;
    let (i, loop_samples) = le_u32(i)?;
    let (i, rate) = le_u32(i)?;
    let (i, sn76489_fb) = le_u16(i)?;
    let (i, sn76489_w) = le_u8(i)?;
    let (i, sn76489_f) = le_u8(i)?;
    let (i, clock_ym2612) = le_u32(i)?;
    let (i, clock_ym2151) = le_u32(i)?;
    let (i, vgm_data_offset) = le_u32(i)?;
    let (i, sega_pcm_clock) = le_u32(i)?;
    let (i, spcm_interface) = le_u32(i)?;
    let (i, clock_rf5c68) = le_u32(i)?;
    let (i, clock_ym2203) = le_u32(i)?;
    let (i, clock_ym2608) = le_u32(i)?;
    let (i, clock_ym2610_b) = le_u32(i)?;
    let (i, clock_ym3812) = le_u32(i)?;
    let (i, clock_ym3526) = le_u32(i)?;
    let (i, clock_y8950) = le_u32(i)?;
    let (i, clock_ymf262) = le_u32(i)?;
    let (i, clock_ymf278_b) = le_u32(i)?;
    let (i, clock_ym271) = le_u32(i)?;
    let (i, clock_ymz280b) = le_u32(i)?;
    let (i, clock_rf5c164) = le_u32(i)?;
    let (i, clock_pwm) = le_u32(i)?;
    let (i, clock_ay8910) = le_u32(i)?;
    let (i, ay8910_chip_type) = le_u8(i)?;
    let (i, ay8910_flag) = le_u16(i)?;
    let (i, volume_modifier) = le_u8(i)?;
    let (i, reserved01) = le_u8(i)?;
    let (i, loop_base) = le_u8(i)?;
    let (i, loop_modifier) = le_u8(i)?;
    let (i, clock_gb_dmg) = le_u32(i)?;
    let (i, clock_nes_apu) = le_u32(i)?;
    let (i, clock_multi_pcm) = le_u32(i)?;
    let (i, clock_upd7759) = le_u32(i)?;
    let (i, clock_okim6258) = le_u32(i)?;
    let (i, okmi6258_flag) = le_u8(i)?;
    let (i, k054539_flag) = le_u8(i)?;
    let (i, c140_chip_type) = le_u8(i)?;
    let (i, reserved02) = le_u8(i)?;
    let (i, clock_okim6295) = le_u32(i)?;
    let (i, clock_k051649) = le_u32(i)?;
    let (i, clock_k054539) = le_u32(i)?;
    let (i, clock_huc6280) = le_u32(i)?;
    let (i, clock_c140) = le_u32(i)?;
    let (i, clock_k053260) = le_u32(i)?;
    let (i, clock_pokey) = le_u32(i)?;
    let (i, clock_qsound) = le_u32(i)?;
    let (i, clock_scsp) = le_u32(i)?;
    let (i, extra_hdr_ofs) = le_u32(i)?;
    let (i, clock_wonder_swan) = le_u32(i)?;
    let (i, clock_vsu) = le_u32(i)?;
    let (i, clock_saa1099) = le_u32(i)?;
    let (i, clock_es5503) = le_u32(i)?;
    let (i, clock_es5506) = le_u32(i)?;
    let (i, es5503_amount_channel) = le_u8(i)?;
    let (i, es5506_amount_channel) = le_u8(i)?;
    let (i, c352_clock_divider) = le_u8(i)?;
    let (i, reserved03) = le_u8(i)?;
    let (i, clock_x1_010) = le_u32(i)?;
    let (i, clock_c352) = le_u32(i)?;
    let (i, clock_ga20) = le_u32(i)?;
    let (i, reserved04) = le_u32(i)?;
    let (i, reserved05) = le_u32(i)?;
    let (i, reserved06) = le_u32(i)?;
    let (i, reserved07) = le_u32(i)?;
    let (i, reserved08) = le_u32(i)?;
    let (i, reserved09) = le_u32(i)?;
    let (i, reserved10) = le_u32(i)?;

    let mut header = VgmHeader::default();

    if version >= 100 {
        header = VgmHeader {
            eof,
            version,
            clock_sn76489,
            clock_ym2413,
            offset_gd3,
            total_samples,
            offset_loop,
            loop_samples,
            ..header
        };
    }
    if version >= 101 {
        header = VgmHeader {
            rate,
            ..header
        };
    }
    if version >= 110 {
        header = VgmHeader {
            sn76489_fb,
            sn76489_w,
            clock_ym2612,
            clock_ym2151,
            ..header
        };
    }
    if version >= 150 {
        header = VgmHeader {
            vgm_data_offset,
            ..header
        };
    }
    if version >= 151 {
        header = VgmHeader {
            sn76489_f,
            sega_pcm_clock,
            spcm_interface,
            clock_rf5c68,
            clock_ym2203,
            clock_ym2608,
            clock_ym2610_b,
            clock_ym3812,
            clock_ym3526,
            clock_y8950,
            clock_ymf262,
            clock_ymf278_b,
            clock_ym271,
            clock_ymz280b,
            clock_rf5c164,
            clock_pwm,
            clock_ay8910,
            ay8910_chip_type,
            ay8910_flag,
            loop_modifier,
            ..header
        };
    }
    if version >= 160 {
        header = VgmHeader {
            volume_modifier,
            reserved01,
            loop_base,
            ..header
        };
    }
    if version >= 161 {
        header = VgmHeader {
            clock_gb_dmg,
            clock_nes_apu,
            clock_multi_pcm,
            clock_upd7759,
            clock_okim6258,
            okmi6258_flag,
            k054539_flag,
            c140_chip_type,
            reserved02,
            clock_okim6295,
            clock_k051649,
            clock_k054539,
            clock_huc6280,
            clock_c140,
            clock_k053260,
            clock_pokey,
            clock_qsound,
            ..header
        };
    }
    if version >= 170 {
        header = VgmHeader {
            extra_hdr_ofs,
            ..header
        };
    }
    if version >= 171 {
        header = VgmHeader {
            clock_scsp,
            clock_wonder_swan,
            clock_vsu,
            clock_saa1099,
            clock_es5503,
            clock_es5506,
            es5503_amount_channel,
            es5506_amount_channel,
            c352_clock_divider,
            reserved03,
            clock_x1_010,
            clock_c352,
            clock_ga20,
            reserved04,
            reserved05,
            reserved06,
            reserved07,
            reserved08,
            reserved09,
            reserved10,
            ..header
        };
    }

    Ok((i, header))
}

///
/// parse_utf16_until_null
///
fn parse_utf16_until_null(i: &[u8]) -> IResult<&[u8], String> {
    let mut string: Vec<u16> = Vec::new();
    let (mut i, mut bytes) = take(2usize)(i)?;
    while bytes != b"\0\0" {
        string.push((bytes[1] as u16) << 8 | (bytes[0] as u16));
        let take = take(2usize)(i)?;
        i = take.0;
        bytes = take.1;
    }
    let string = match String::from_utf16(&string) {
        Ok(string) => string,
        Err(_) => String::from("")
    };

    Ok((i, string))
}

///
/// parse_vgm_gd3
///
fn parse_vgm_gd3(i: &[u8]) -> IResult<&[u8], Gd3> {
    let (i, _) = tag("Gd3 ")(i)?;
    let (i, _) = take(4usize)(i)?; // version
    let (i, _) = take(4usize)(i)?; // length

    let (i, track_name) = parse_utf16_until_null(i)?;
    let (i, track_name_j) = parse_utf16_until_null(i)?;
    let (i, game_name) = parse_utf16_until_null(i)?;
    let (i, game_name_j) = parse_utf16_until_null(i)?;
    let (i, system_name) = parse_utf16_until_null(i)?;
    let (i, system_name_j) = parse_utf16_until_null(i)?;
    let (i, track_author) = parse_utf16_until_null(i)?;
    let (i, track_author_j) = parse_utf16_until_null(i)?;
    let (i, date) = parse_utf16_until_null(i)?;
    let (i, converted) = parse_utf16_until_null(i)?;

    Ok((i, Gd3 {
        track_name,
        track_name_j,
        game_name,
        game_name_j,
        system_name,
        system_name_j,
        track_author,
        track_author_j,
        date,
        converted,
    }))
}

///
/// parse_vgm_meta
///
pub(crate) fn parse_vgm_meta(vgmdata: &[u8]) -> Result<(VgmHeader, Gd3), &'static str> {
    let header = match parse_vgm_header(&vgmdata[..255]) {
        Ok((_, header)) => header,
        Err(_) => {
            return Err("vgm header parse error.")
        }
    };
    let gd3 = match parse_vgm_gd3(&vgmdata[(0x14 + header.offset_gd3 as usize)..]) {
        Ok((_, gd3)) => gd3,
        Err(_) => Gd3::default() // blank values
    };

    Ok((header, gd3))
}

///
/// Jsonlize
///
pub(crate) trait Jsonlize : serde::Serialize {
    fn get_json(&self) -> String {
        match serde_json::to_string(&self) {
            Ok(json) => json,
            Err(_) => String::from("")
        }
    }
}

impl Jsonlize for VgmHeader { }
impl Jsonlize for Gd3 { }

#[cfg(test)]
mod tests {
    use super::parse_vgm_meta;
    use super::Jsonlize;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_1() {
        parse("../docs/vgm/ym2612.vgm")
    }

    fn parse(filepath: &str) {
        // load sn76489 vgm file
        let mut file = File::open(filepath).unwrap();
        let mut buffer = Vec::new();
        let _ = file.read_to_end(&mut buffer).unwrap();
        let (header, gd3) = parse_vgm_meta(&buffer).unwrap();

        println!("{:#?}", header);
        println!("{:#?}", gd3);
        println!("{:#?}", header.get_json());
        println!("{:#?}", gd3.get_json());
    }
}
