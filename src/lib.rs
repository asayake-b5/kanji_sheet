use std::collections::HashMap;

use arg_parsing::kanji_to_hexcode;
use pages::Pages;
use pdf_creation::kanji_to_png;

pub mod arg_parsing;
pub mod pages;
pub mod pdf_creation;
pub mod worker;

//FIXME remove clone later
#[derive(Debug, Clone)]
pub struct Globals {
    pub _stroke_map: HashMap<String, usize>,
    pub svgs: HashMap<String, Vec<u8>>,
}

#[derive(PartialEq, Eq)]
pub enum KanjiToPngErrors {
    FileNotFound,
    Undefined,
}

pub fn create_pages(
    kanjis: &str,
    add_blank: u16,
    add_grid: u16,
    data: Globals,
) -> (Pages, Vec<char>) {
    let mut pages = Pages::default();
    pages.add_page();
    let mut skipped_kanji = Vec::<char>::with_capacity(10);

    for kanji in kanjis.chars() {
        if let Err(e) = kanji_to_png(
            &mut pages,
            // &kanji_to_filename(kanji),
            &kanji_to_hexcode(kanji),
            add_blank,
            add_grid,
            data.clone(),
        ) {
            if e == KanjiToPngErrors::FileNotFound {
                skipped_kanji.push(kanji);
            }
        }
    }
    (pages, skipped_kanji)
}

pub fn do_csv() -> HashMap<String, usize> {
    let mut ret: HashMap<String, usize> = HashMap::new();
    let mut opt = usvg::Options::default();
    opt.fontdb.load_system_fonts();
    for file in std::fs::read_dir("./assets/svg").unwrap().flatten() {
        let svg_data = std::fs::read(file.path()).unwrap();
        let rtree = usvg::Tree::from_data(&svg_data, &opt.to_ref()).unwrap();
        let len = rtree
            .root()
            .descendants()
            .filter(|descendant| {
                if let usvg::NodeKind::Path(_) = *descendant.borrow() {
                    true
                } else {
                    false
                }
            })
            .count();
        ret.insert(
            file.path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            len,
        );
    }
    ret
}

pub fn do_svgs() -> HashMap<String, Vec<u8>> {
    let mut ret: HashMap<String, Vec<u8>> = HashMap::new();
    for file in std::fs::read_dir("./assets/svg").unwrap().flatten() {
        let svg_data = std::fs::read(file.path()).unwrap();
        ret.insert(
            file.path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            svg_data,
        );
    }
    ret
}
