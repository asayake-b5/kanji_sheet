use image::{DynamicImage, ImageBuffer, Rgba};
use usvg::{Color, Opacity, Stroke, StrokeMiterlimit, StrokeWidth};

#[derive(Eq, PartialEq, Debug)]
pub enum Overflow {
    None,
    ChangedLine,
    ChangedPage,
}

pub struct Pages {
    x: u32,
    y: u32,
    layer: usize,
    imgs: Vec<DynamicImage>,
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
        self.imgs.push(
            image::load_from_memory_with_format(Pages::BYTES, image::ImageFormat::Png).unwrap(),
        );
    }

    pub fn save_pages(&self, list: &str) {
        let blank_sheet =
            image::load_from_memory_with_format(Pages::BYTES, image::ImageFormat::Png).unwrap();

        std::fs::create_dir_all(&format!("out/{}", list)).unwrap();
        for (i, img) in self.imgs.iter().enumerate() {
            if img != &blank_sheet {
                img.save(&format!("out/{}/page-{}.png", list, i)).unwrap();
            }
        }
    }

    pub fn draw_svg(&mut self, grid: &DynamicImage, svg_img: &ImageBuffer<Rgba<u8>, &[u8]>) {
        image::imageops::overlay(&mut self.imgs[self.layer], grid, self.x, self.y);
        image::imageops::overlay(&mut self.imgs[self.layer], svg_img, self.x + 3, self.y + 3);
        self.next();
    }

    pub fn draw_clean_squares(&mut self, i: u32) {
        let blank =
            image::load_from_memory_with_format(Pages::BLANK_BYTES, image::ImageFormat::Png)
                .unwrap();
        for _ in 0..i {
            image::imageops::overlay(&mut self.imgs[self.layer], &blank, self.x, self.y);
            self.next();
        }
    }

    pub fn fill_line(&mut self, grid: &DynamicImage, svg_img: &ImageBuffer<Rgba<u8>, &[u8]>) {
        while self.peek_next() == Overflow::None {
            self.draw_svg(grid, svg_img);
        }
        self.draw_svg(grid, svg_img);
    }

    pub fn draw_full_opaque(&mut self, svg_data: &[u8], i: u32) {
        // let blank = image::load_from_memory_with_format(BLANK_BYTES, image::ImageFormat::Png).unwrap();
        let mut opt = usvg::Options::default();
        opt.fontdb.load_system_fonts();
        let tree = usvg::Tree::from_data(svg_data, &opt.to_ref()).unwrap();
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
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
        resvg::render(
            &tree,
            usvg::FitTo::Original,
            tiny_skia::Transform::default(),
            pixmap.as_mut(),
        )
        .unwrap();
        let svg_img =
            image::ImageBuffer::from_raw(Pages::VIEWBOX_U, Pages::VIEWBOX_U, pixmap.data())
                .unwrap();
        for _ in 0..i {
            image::imageops::overlay(&mut self.imgs[self.layer], &svg_img, self.x + 3, self.y + 3);

            self.next();
        }
    }

    pub fn new_line(&mut self, gap: u32) {
        self.x = Pages::X_OFFSET;
        self.y += gap;
        if self.y > Pages::HEIGHT {
            self.add_page();
            self.layer += 1;
            self.y = Pages::Y_OFFSET;
        }
    }
}

impl Default for Pages {
    fn default() -> Self {
        Self {
            x: Pages::X_OFFSET,
            y: Pages::Y_OFFSET,
            layer: 0,
            imgs: Vec::with_capacity(8),
        }
    }
}
