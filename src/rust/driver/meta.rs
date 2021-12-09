// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka

///
/// Jsonlize
///
pub(crate) trait Jsonlize: serde::Serialize {
    fn get_json(&self) -> String {
        match serde_json::to_string(&self) {
            Ok(json) => json,
            Err(_) => String::from(""),
        }
    }
}
