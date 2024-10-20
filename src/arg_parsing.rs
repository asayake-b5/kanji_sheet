// use std::str::FromStr;
use std::fmt::Write;

/// Returns the hexadecimal UTF-8 code for the kanji, i.e. what appears before .svg in the KanjiVG filenames.
/// ```
/// # use kanji_practice_sheet::arg_parsing::kanji_to_hexcode;
/// assert_eq!(kanji_to_hexcode('淌'), "06dcc");
/// ```
pub fn kanji_to_hexcode(c: char) -> String {
    if c == '𦥑' {
        return String::from("26951");
    }
    let c_u16 = c as u16;
    let c_slice = c_u16.to_be_bytes();

    let mut path_string = String::from("0");
    c_slice
        .iter()
        .for_each(|x| write!(path_string, "{:02x}", x).unwrap());
    path_string
}

pub fn kanji_to_filename(c: char) -> String {
    let path_string = kanji_to_hexcode(c);
    // let path_string =
    format!("assets/svg/{}.svg", path_string)
}
