use kanji_practice_sheet::{arg_parsing::kanji_to_filename, pdf_creation::kanji_to_png};

fn main() {
    let now = std::time::Instant::now();
    kanji_to_png(&kanji_to_filename('家'));
    println!("{:?}", now.elapsed());
}
