use std::borrow::Borrow;

use kanji_practice_sheet::arg_parsing::kanji_to_filename;
use usvg::PathData;

#[test]
fn test_get_path() {
    let kanji_1 = kanji_to_filename('𦥑');
    let kanji_2 = kanji_to_filename('淌');
    assert_eq!(kanji_1, String::from("assets/svg/26951.svg"));
    assert_eq!(kanji_2, String::from("assets/svg/06dcc.svg"));
}

#[test]
fn test_stroke_order_correctness() {
    let paths = std::fs::read_dir("./assets/svg").unwrap();
    for path in paths {
        let mut u = 1;

        let path = path.unwrap().path();
        dbg!(&path);
        let svg_data = std::fs::read(path).unwrap();
        let mut opt = usvg::Options::default();
        opt.fontdb.load_system_fonts();
        let rtree = usvg::Tree::from_data(&svg_data, &opt.to_ref()).unwrap();

        for node in rtree.root().descendants() {
            if let usvg::NodeKind::Path(ref path) = *node.borrow() {
                let stuff = &path.id.split('-').last().unwrap()[1..];
                let data: &PathData = path.data.borrow();
                assert!(!data.is_empty());
                let new = stuff.parse::<u8>().unwrap();
                assert_eq!(new, u);
                u += 1;
            }
        }
    }
}

#[test]
fn test_validity_of_files() {
    let paths = std::fs::read_dir("./assets/svg").unwrap();
    for path in paths {
        let path = path.unwrap().path();
        dbg!(&path);
        let svg_data = std::fs::read(path).unwrap();

        let mut opt = usvg::Options::default();
        opt.fontdb.load_system_fonts();
        let _ = usvg::Tree::from_data(&svg_data, &opt.to_ref()).unwrap();
    }
}
