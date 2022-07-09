use usvg::{Align, AspectRatio, Color, Opacity, Rect, Size, Stroke, StrokeMiterlimit, StrokeWidth};

use crate::pages::Pages;

pub fn kanji_to_png(pages: &mut Pages, path: &str) {
    let grid =
        image::load_from_memory_with_format(Pages::BYTES_GRID, image::ImageFormat::Png).unwrap();
    // let svg_data = std::fs::read("assets/svg/04eee.svg").unwrap();
    let svg_data = std::fs::read(path).unwrap();
    let mut opt = usvg::Options::default();
    opt.fontdb.load_system_fonts();
    let rtree = usvg::Tree::from_data(&svg_data, &opt.to_ref()).unwrap();

    // These unwraps should be okay, we're using handwritten stuff anyway
    let tree2 = usvg::Tree::create(usvg::Svg {
        size: Size::new(Pages::VIEWBOX_F, Pages::VIEWBOX_F).unwrap(),
        view_box: usvg::ViewBox {
            rect: Rect::new(0.0, 0.0, Pages::VIEWBOX_F, Pages::VIEWBOX_F).unwrap(),
            aspect: AspectRatio {
                // ??? to all three
                defer: false,
                align: Align::XMidYMid,
                slice: true,
            },
        },
    });

    pages.draw_full_opaque(&svg_data, 1);

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
                image::ImageBuffer::from_raw(Pages::VIEWBOX_U, Pages::VIEWBOX_U, pixmap.data())
                    .unwrap();
            // let (x, y, layer) = calculate_top_left(*n);
            // image::imageops::overlay(&mut imgs[layer], &grid, x, y);
            // image::imageops::overlay(&mut pages.imgs[0], &svg_img, 3, 3);
            pages.draw_svg(&grid, &svg_img);
        }
    }

    let pixmap_size = tree2.svg_node().size.to_screen_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(
        &tree2,
        usvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();
    let svg_img =
        image::ImageBuffer::from_raw(Pages::VIEWBOX_U, Pages::VIEWBOX_U, pixmap.data()).unwrap();
    pages.fill_line(&grid, &svg_img);

    pages.draw_clean_squares(Pages::N_SQUARE_PER_LINE);
    pages.new_line(20);
}

pub fn create_pdf(pages: &Pages, list: &str) {
    let font_family = genpdf::fonts::from_files("./assets/font/", "Courier", None).unwrap();
    let mut doc = genpdf::Document::new(font_family);
    doc.set_title(list);
    for img in &pages.imgs {
        let rgb8 = image::DynamicImage::ImageRgb8(img.to_rgb8());
        doc.push(
            genpdf::elements::Image::from_dynamic_image(rgb8)
                .unwrap()
                .with_dpi(160.0),
        );
    }
    doc.render_to_file(&format!("out/{}.pdf", list)).unwrap();
}
