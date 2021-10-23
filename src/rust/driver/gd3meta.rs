// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
use nom::bytes::complete::{tag, take};
use nom::IResult;

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
    pub converted: String,
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
        Err(_) => String::from(""),
    };

    Ok((i, string))
}

///
/// parse_vgm_gd3
///
pub fn parse_vgm_gd3(i: &[u8]) -> IResult<&[u8], Gd3> {
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

    Ok((
        i,
        Gd3 {
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
        },
    ))
}
