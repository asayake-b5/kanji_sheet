use usvg::{Align, AspectRatio, Color, Opacity, Rect, Size, Stroke, StrokeMiterlimit, StrokeWidth};

const BYTES: &[u8] = include_bytes!("../assets/kanji_sheet.png");
// TODO will be changed, in the future
// also, need like a neew sheet with everything 109 apart
const X_OFFSET: u32 = 19;
const Y_OFFSET: u32 = 17;

const VIEWBOX_U: u32 = 109;
const VIEWBOX_F: f64 = 109.0;

const N_SQUARE_PER_PAGE: u32 = 165;
const N_SQUARE_PER_LINE: u32 = 11;

/// slot => (x, y, z (layer))
pub fn calculate_top_left(n: u32) -> (u32, u32, u32) {
    let layer = n / N_SQUARE_PER_PAGE;
    let n2 = n - N_SQUARE_PER_PAGE * layer;
    let y_n = n2 / N_SQUARE_PER_LINE;
    let x_n = n2 % N_SQUARE_PER_LINE;

    // let x = X_OFFSET + x_n * VIEWBOX_U;
    // let y = Y_OFFSET + y_n * VIEWBOX_U;
    let x = X_OFFSET + x_n * 113;
    let y = Y_OFFSET + y_n * 113;

    (x, y, layer)
}

pub fn kanji_to_png(path: &str) {
    let mut img = image::load_from_memory_with_format(BYTES, image::ImageFormat::Png).unwrap();

    // let svg_data = std::fs::read("assets/svg/04eee.svg").unwrap();
    let svg_data = std::fs::read(path).unwrap();
    let mut opt = usvg::Options::default();
    opt.fontdb.load_system_fonts();
    let rtree = usvg::Tree::from_data(&svg_data, &opt.to_ref()).unwrap();

    // These unwraps should be okay, we're using handwritten stuff anyway
    let tree2 = usvg::Tree::create(usvg::Svg {
        size: Size::new(VIEWBOX_F, VIEWBOX_F).unwrap(),
        view_box: usvg::ViewBox {
            rect: Rect::new(0.0, 0.0, VIEWBOX_F, VIEWBOX_F).unwrap(),
            aspect: AspectRatio {
                // ??? to all three
                defer: false,
                align: Align::XMidYMid,
                slice: true,
            },
        },
    });

    let mut i = 0;

    for mut node in rtree.root().descendants() {
        tree2.root().append(node.make_copy());

        if let usvg::NodeKind::Path(ref mut _path) = *node.borrow_mut() {
            if let usvg::NodeKind::Path(ref mut path2) =
                *tree2.root().last_child().unwrap().borrow_mut()
            {
                path2.stroke = Some(Stroke {
                    paint: usvg::Paint::Color(Color::new_rgb(138, 152, 155)), // Change the paint per stroke???
                    dasharray: None,                                          // WHAT EVEN IS THIS
                    dashoffset: 0.0,                                          // ??????
                    miterlimit: StrokeMiterlimit::default(),                  // should be ok??
                    opacity: Opacity::new(0.95), // FINALLY SOMETHING I UNDERSTAND
                    width: StrokeWidth::new(4.0),
                    linecap: usvg::LineCap::Round,
                    linejoin: usvg::LineJoin::Round,
                });
            }
            let pixmap_size = tree2.svg_node().size.to_screen_size();
            let mut pixmap =
                tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
            resvg::render(
                &tree2,
                usvg::FitTo::Original,
                tiny_skia::Transform::default(),
                pixmap.as_mut(),
            )
            .unwrap();
            let svg_img =
                image::ImageBuffer::from_raw(VIEWBOX_U, VIEWBOX_U, pixmap.data()).unwrap();
            let (x, y, _) = calculate_top_left(i);
            image::imageops::overlay(&mut img, &svg_img, x, y);

            i += 1;
        }
    }
    img.save("test.png").unwrap();
}

pub fn create_pdf() {}
