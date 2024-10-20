use image::{DynamicImage, ImageBuffer, Rgba};
use usvg::{Color, Opacity, Stroke, StrokeMiterlimit, StrokeWidth};

use crate::KanjiToPngErrors;

#[derive(Eq, PartialEq, Debug)]
pub enum Overflow {
    None,
    ChangedLine,
    ChangedPage,
}

pub enum BgType {
    Blank,
    Grid,
}

pub struct Pages {
    x: u32,
    y: u32,
    layer: usize,
    pub imgs: Vec<DynamicImage>,
    blank_line: DynamicImage,
    blank_sheet: DynamicImage,
    pub grid: DynamicImage,
    grid_line: DynamicImage,
    pub opt: usvg::Options,
}

impl Pages {
    pub const BYTES: &'static [u8] = include_bytes!("../assets/kanji_sheet.png");
    pub const BYTES_GRID: &'static [u8] = include_bytes!("../assets/box_dotted.png");
    pub const BLANK_BYTES: &'static [u8] = include_bytes!("../assets/box_empty.png");
    pub const WIDTH: u32 = 1_303;
    pub const HEIGHT: u32 = 1_887;
    pub const X_OFFSET: u32 = 60;
    pub const Y_OFFSET: u32 = 60;
    pub const VIEWBOX_U: u32 = 109;
    pub const VIEWBOX_F: f64 = 109.0;

    pub const N_SQUARE_PER_PAGE: u32 = 165;
    pub const N_SQUARE_PER_LINE: u32 = 10;

    //TODO test this function
    fn next(&mut self) {
        self.x += 114;
        // if out of bounds change the bounds
        if self.x + 114 > Pages::WIDTH - Pages::X_OFFSET {
            // new line
            self.x = Pages::X_OFFSET;
            self.y += 114;
        }
        if self.y + 114 > Pages::HEIGHT - Pages::Y_OFFSET {
            // new page
            self.x = Pages::X_OFFSET;
            self.y = Pages::Y_OFFSET;
            self.add_page();
            self.layer += 1;
        }
    }

    fn peek_next(&self) -> Overflow {
        let mut o = Overflow::None;
        let x_temp = self.x + 114;
        let mut y_temp = self.y;

        if x_temp + 114 > Pages::WIDTH - Pages::X_OFFSET {
            y_temp += 114;
            o = Overflow::ChangedLine;
        }
        if y_temp + 114 > Pages::HEIGHT - Pages::Y_OFFSET {
            o = Overflow::ChangedPage;
        }

        o
    }

    pub fn add_page(&mut self) {
        self.imgs.push(self.blank_sheet.clone());
    }

    pub fn draw_svg(&mut self, svg_img: &ImageBuffer<Rgba<u8>, &[u8]>) {
        image::imageops::overlay(&mut self.imgs[self.layer], &self.grid, self.x, self.y);
        image::imageops::overlay(&mut self.imgs[self.layer], svg_img, self.x + 3, self.y + 3);
        self.next();
    }

    fn draw_n_full_grid(&mut self, n: u16) {
        for _ in 0..n {
            image::imageops::overlay(&mut self.imgs[self.layer], &self.grid_line, self.x, self.y);
            self.new_line(114);
        }
    }

    fn draw_n_full_blank(&mut self, n: u16) {
        for _ in 0..n {
            image::imageops::overlay(&mut self.imgs[self.layer], &self.blank_line, self.x, self.y);
            self.new_line(114);
        }
    }

    pub fn draw_n_full_lines(&mut self, bg_type: BgType, n: u16) {
        match bg_type {
            BgType::Blank => self.draw_n_full_blank(n),
            BgType::Grid => self.draw_n_full_grid(n),
        };
    }

    pub fn fill_line(&mut self, svg_img: &ImageBuffer<Rgba<u8>, &[u8]>) {
        while self.peek_next() == Overflow::None {
            self.draw_svg(svg_img);
        }
        self.draw_svg(svg_img);
    }

    pub fn draw_full_opaque(&mut self, svg_data: &[u8], i: u32) -> Result<(), KanjiToPngErrors> {
        let tree = usvg::Tree::from_data(svg_data, &self.opt.to_ref())
            .map_err(|_| KanjiToPngErrors::UnlikelyError)?;
        for mut node in tree.root().descendants() {
            if let usvg::NodeKind::Path(ref mut path) = *node.borrow_mut() {
                path.stroke = Some(Stroke {
                    paint: usvg::Paint::Color(Color::new_rgb(0, 0, 0)), // Change the paint per stroke???
                    dasharray: None,                                    // WHAT EVEN IS THIS
                    dashoffset: 0.0,                                    // ??????
                    miterlimit: StrokeMiterlimit::default(),            // should be ok??
                    opacity: Opacity::new(1.0), // FINALLY SOMETHING I UNDERSTAND
                    width: StrokeWidth::new(4.0),
                    linecap: usvg::LineCap::Round,
                    linejoin: usvg::LineJoin::Round,
                });
            }
        }
        let pixmap_size = tree.svg_node().size.to_screen_size();
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
            .ok_or(KanjiToPngErrors::UnlikelyError)?;
        resvg::render(
            &tree,
            usvg::FitTo::Original,
            tiny_skia::Transform::default(),
            pixmap.as_mut(),
        )
        .ok_or(KanjiToPngErrors::UnlikelyError)?;
        let svg_img =
            image::ImageBuffer::from_raw(Pages::VIEWBOX_U, Pages::VIEWBOX_U, pixmap.data())
                .ok_or(KanjiToPngErrors::UnlikelyError)?;
        for _ in 0..i {
            image::imageops::overlay(&mut self.imgs[self.layer], &svg_img, self.x + 3, self.y + 3);

            self.next();
        }
        Ok(())
    }

    pub fn new_line(&mut self, gap: u32) {
        self.x = Pages::X_OFFSET;
        self.y += gap;
        if self.y + 114 > Pages::HEIGHT - Pages::Y_OFFSET {
            self.add_page();
            self.layer += 1;
            self.y = Pages::Y_OFFSET;
        }
    }

    fn prepare(bg_type: &DynamicImage) -> DynamicImage {
        let mut image = DynamicImage::new_rgba8(Pages::WIDTH, Pages::VIEWBOX_U + 30);
        let mut x = 0;

        for _ in 0..Pages::N_SQUARE_PER_LINE {
            image::imageops::overlay(&mut image, bg_type, x, 0);
            x += 114;
        }

        image
    }

    pub fn new() -> Result<Self, KanjiToPngErrors> {
        //None of these errors should realistically happen
        let blank =
            image::load_from_memory_with_format(Pages::BLANK_BYTES, image::ImageFormat::Png)
                .map_err(|_| KanjiToPngErrors::UnlikelyError)?;
        let blank_sheet =
            image::load_from_memory_with_format(Pages::BYTES, image::ImageFormat::Png)
                .map_err(|_| KanjiToPngErrors::UnlikelyError)?;
        let grid = image::load_from_memory_with_format(Pages::BYTES_GRID, image::ImageFormat::Png)
            .map_err(|_| KanjiToPngErrors::UnlikelyError)?;

        let mut opt = usvg::Options::default();
        opt.fontdb.load_system_fonts();

        let blank_line = Pages::prepare(&blank);
        let grid_line = Pages::prepare(&grid);

        Ok(Self {
            opt,
            grid,
            blank_sheet,
            blank_line,
            grid_line,
            x: Pages::X_OFFSET,
            y: Pages::Y_OFFSET,
            layer: 0,
            imgs: Vec::with_capacity(8),
        })
    }
}
