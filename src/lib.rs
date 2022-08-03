use std::{collections::HashMap, net::TcpListener};

use arg_parsing::kanji_to_filename;
use pages::Pages;
use pdf_creation::kanji_to_png;

pub mod arg_parsing;
pub mod pages;
pub mod pdf_creation;

#[derive(PartialEq, Eq)]
pub enum KanjiToPngErrors {
    FileNotFound,
    Undefined,
}

pub fn find_free_port() -> Option<u16> {
    (8000..55000).find(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok())
}

pub async fn launch_browser(url: &str) {
    std::thread::sleep(std::time::Duration::from_millis(300));
    if webbrowser::open(url).is_ok() {}
}

pub fn create_pages(kanjis: &str, add_blank: u16, add_grid: u16) -> (Pages, Vec<char>) {
    let mut pages = Pages::default();
    pages.add_page();
    let mut skipped_kanji = Vec::<char>::with_capacity(10);

    for kanji in kanjis.chars() {
        if let Err(e) = kanji_to_png(&mut pages, &kanji_to_filename(kanji), add_blank, add_grid) {
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
