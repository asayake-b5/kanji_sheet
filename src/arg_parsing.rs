// use std::str::FromStr;
use std::fmt::Write;

/// Returns the hexadecimal UTF-8 code for the kanji, i.e. what appears before .svg in the KanjiVG filenames.
/// ```
/// # use kanji_practice_sheet::arg_parsing::kanji_to_hexcode;
/// assert_eq!(kanji_to_hexcode('淌'), "06dcc");
/// ```
pub fn kanji_to_hexcode(c: char) -> String {
    let c_u32 = c as u32;
    let c_slice = c_u32.to_be_bytes();
    let mut path_string = String::new();
    // Ignore the first byte, we don't need it
    c_slice
        .iter()
        .skip(1)
        .for_each(|x| write!(path_string, "{:x}", x).unwrap());
    path_string
}

pub fn kanji_to_filename(c: char) -> String {
    let path_string = kanji_to_hexcode(c);
    // let path_string =
    format!("assets/svg/{}.svg", path_string)
}

// use svg::{
//     node::{
//         element::{path::Data, tag::Type},
//         Value,
//     },
//     parser::Event,
// };

// const MAX_ELEMENT_STACK: usize = 11;

// // TODO assign colors, have a 30 list of defined thing, let people use their own as optional argument, and if not enough return an error
// pub fn assign_colors() {
//     todo!()
// }

// If you use kanjivg-*-main.zip, this option will always be None. If you use the package including variants, wrap the case-sensitive string of the variant you want.
// Do keep in mind most kanjis won't feature variants, and that some are present in as little as 1 or 2 kanji, so you should really be sure of what you want to do to avoid errors.
// For most use cases, None is recommended.
// type KanjiVariant<'a> = Option<&'a str>;

//TODO change this name
// fn element_do_something(element: &Element) -> Vec<&KanjiPath> {
//     let mut vec = Vec::<&KanjiPath>::with_capacity(5);
//     for child in element.children().unwrap() {
//         match child {
//             Element::Path(path) => {
//                 vec.push(path);
//             }
//             Element::Element(_) => vec.extend(element_do_something(child)),
//             Element::None => {}
//         }
//     }
//     vec
// }
// pub fn to_group_path(kanji: &'_ Kanji) -> KanjiPathList<'_> {
//     let mut paths = Vec::<KanjiPathListEntry>::with_capacity(30);

//     for group in kanji.root_element().children().unwrap() {
//         match group {
//             Element::Element(_) => {
//                 paths.push(KanjiPathListEntry(
//                     group.label().unwrap_or_else(|| String::from("")),
//                     element_do_something(group),
//                 ));
//             }
//             // In some cases (like kanas), there is no subgroup and everything is in a flat
//             // TODO can we do it better?
//             Element::Path(path) => {
//                 paths.push(KanjiPathListEntry(
//                     group.label().unwrap_or_else(|| String::from("")),
//                     vec![path],
//                 ));
//             }
//             Element::None => {}
//         }
//     }
//     KanjiPathList(paths)
// }

// fn parse_stroke(text: &str, matrix: String) -> Stroke {
//     let text = str::parse::<u8>(text).unwrap();
//     // matrix.remove_matches("matrix(");
//     // matrix.remove_matches(")");
//     let mut values = Vec::<f64>::with_capacity(6);
//     for a in matrix
//         .trim_end_matches(')')
//         .trim_start_matches("matrix(")
//         .split_whitespace()
//     {
//         values.push(str::parse::<f64>(a).unwrap());
//     }
//     // println!("{:?}", values);
//     Stroke {
//         text,
//         // TODO this not good !
//         matrix: values.try_into().unwrap(),
//     }
// }

// fn parse_svg_path(instructions: &str) -> Data {
//     Data::parse(instructions).unwrap()
// }

// fn parse_kanji_path(attributes: &HashMap<String, Value>) -> KanjiPath {
//     // TODO call a parsing function here and stuff
//     // TODO v ??
//     dbg!(attributes);
//     let _null_value = Value::from("");
//     let instructions = attributes.get("d").unwrap();
//     let symbol = attributes.get("kvg:type");
//     let label = attributes.get("id").unwrap();
//     let instructions = parse_svg_path(instructions);
//     KanjiPath {
//         label: label.to_string(),
//         symbol: symbol.map(|symbol| symbol.to_string()),
//         instructions,
//     }
// }

// pub fn pignon(path: &str) -> Kanji {
//     // Should be much more than enough
//     // TODO remove capacity, test all svgs, check highest capacity?
//     // let mut element_stack = Vec::<Element>::with_capacity(10);
//     let mut element_stack = Vec::<Element>::with_capacity(MAX_ELEMENT_STACK);
//     let mut text_stack = Vec::<String>::with_capacity(2);
//     let mut content = String::new();
//     let mut parser = svg::open(path, &mut content).unwrap();
//     let mut kanji = Kanji {
//         height: 0,
//         width: 0,
//         root_element: Element::None,
//         matrix: Vec::<Stroke>::with_capacity(50),
//     };
//     let mut done_stroke_path = false;

//     // Parse <svg xmlns....>
//     // if let Some(Event::Tag(_, _, attributes)) = parser.nth(5) {
//     //     kanji.height = u32::from_str(attributes.get("height").unwrap()).unwrap();
//     //     kanji.width = u32::from_str(attributes.get("width").unwrap()).unwrap();
//     //     // println!("ifl {}{:?}", tag, attributes);
//     // }
//     kanji.height = 109;
//     kanji.width = 109;

//     for event in parser {
//         if let Event::Tag(tag, tag_type, attributes) = event {
//             match tag_type {
//                 Type::Start => {
//                     if tag == "g" {
//                         let element = Element::Element(Box::new(KanjiElement {
//                             label: attributes.get("id").unwrap().to_string(),
//                             children: Vec::new(),
//                         }));
//                         element_stack.push(element);
//                         // println!("start, {}{:?}", tag, attributes)
//                     } else if tag == "text" {
//                         text_stack.push(attributes.get("transform").unwrap().to_string());
//                     }
//                 }
//                 Type::End => {
//                     if tag == "g" {
//                         let element = element_stack.pop().unwrap();
//                         if let Some(Element::Element(lm)) = element_stack.last_mut() {
//                             lm.children.push(element);
//                         } else {
//                             //TODO there's gotta be a better way lol
//                             if !done_stroke_path {
//                                 kanji.root_element = element
//                                     .children_vec()
//                                     .unwrap()
//                                     .get_mut(0)
//                                     .map(mem::take)
//                                     .unwrap_or_default();
//                                 done_stroke_path = true;
//                             }
//                         }
//                     } else if tag == "text" {
//                         let text = text_stack.pop().unwrap();
//                         let matrix = text_stack.pop().unwrap();
//                         let stroke = parse_stroke(&text, matrix);
//                         kanji.matrix.push(stroke);
//                         // println!("{:?}", attributes);
//                     }
//                 }
//                 Type::Empty => {
//                     let kanji_struct = parse_kanji_path(&attributes);
//                     let element = Element::Path(kanji_struct);
//                     if let Some(Element::Element(lm)) = element_stack.last_mut() {
//                         lm.children.push(element);
//                     }
//                 }
//             }
//             // // let data = Data::parse(data).unwrap();
//             // //     for command in data.iter() {
//             // //         match *command {
//             // //             Command::Move(..) => println!("Move!"),
//             // //             Command::Line(..) => println!("Line!"),
//             // //             _ => {}
//             // //         }
//             // //     }
//         } else if let Event::Text(text) = event {
//             text_stack.push(text.to_string());
//         }
//         // println!("{:?}", element_stack);
//         // println!("{:?}", text_stack);
//     }
//     // println!("{}", element_stack.capacity());
//     // println!("{}", text_stack.capacity());
//     // println!("{:?}", kanji);
//     // let data =
//     //     Data::parse("M56.12,52.12c5.64,0.81,18.99,22.02,31.03,33.71c2.35,2.29,5.22,4.66,7.84,6.11")
//     //         .unwrap();
//     // for command in data.iter() {
//     //     println!("{:?}", *command);
//     //     match *command {
//     //         Command::Move(..) => println!("Move!"),
//     //         Command::Line(..) => println!("Line!"),
//     //         _ => {}
//     //     }
//     // }
//     kanji
// }

// //      Root   -> [Matrix]
// //  Element Element
// // Ele Ele  Ele Ele
// // P P P P  P P P P

// // [path1 path2 path3 path4] (all black)
// // or
// // [(path1, color1), (path2, color1), (path3, color1), (path4, color2), (path5, color3) ... ]

// #[derive(Debug)]
// struct KanjiPathListEntry<'a>(String, Vec<&'a KanjiPath>);
// #[derive(Debug)]
// pub struct KanjiPathList<'a>(Vec<KanjiPathListEntry<'a>>);

// #[derive(Debug, Clone)]
// // TODO: Change String to &str
// pub struct KanjiElement {
//     label: String,
//     children: Vec<Element>,
// }
// #[derive(Debug, Clone)]
// pub struct KanjiPath {
//     label: String,
//     symbol: Option<String>,
//     instructions: Data,
// }
// #[derive(Debug)]
// pub struct Stroke {
//     matrix: [f64; 6],
//     text: u8,
// }
// #[derive(Debug, Clone)]
// pub enum Element {
//     Element(Box<KanjiElement>),
//     Path(KanjiPath),
//     None,
// }
// #[derive(Debug)]
// pub struct Kanji {
//     width: u32,
//     height: u32,
//     pub root_element: Element,
//     matrix: Vec<Stroke>,
// }

// impl Default for Kanji {
//     fn default() -> Self {
//         Self {
//             width: 0,
//             height: 0,
//             root_element: Element::None,
//             matrix: Vec::new(),
//         }
//     }
// }

// impl Kanji {
//     pub fn root_element(&self) -> &Element {
//         &self.root_element
//     }
// }

// impl Element {
//     pub fn children(&self) -> Result<&[Element], ()> {
//         if let Element::Element(test) = self {
//             // println!("{:?}", &test.children);
//             return Ok(&test.children);
//         }
//         Err(())
//     }
//     fn children_vec(self) -> Result<Vec<Element>, ()> {
//         if let Element::Element(test) = self {
//             return Ok(test.children);
//         }
//         Err(())
//     }
//     pub fn label(&self) -> Option<String> {
//         match self {
//             Element::Element(ele) => Some(ele.label.clone()),
//             Element::Path(path) => Some(path.label.clone()),
//             Element::None => None,
//         }
//     }
// }

// impl Default for Element {
//     fn default() -> Self {
//         Self::None
//     }
// }
