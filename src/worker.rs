use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use relm4::{ComponentSender, Worker};

use crate::{create_pages, do_csv, do_svgs, pdf_creation::create_pdf, Globals};

#[derive(Debug)]
pub enum AppMsg {
    SendMessage(String, bool),
    End,
}

#[derive(Debug)]
pub enum AsyncHandlerMsg {
    Start(KanjiRequest),
}

pub struct AsyncHandler;

#[derive(Debug)]
pub struct KanjiRequest {
    pub kanjis: String,
    pub extra_grid: u16,
    pub extra_blank: u16,
    pub pdf: bool,
    pub png: bool,
    pub _opt_space: bool,
    pub _coloring: Option<String>, // TODO can serde do this as enum?
}

fn process(req: KanjiRequest, data: Globals, timestamp: u128) -> String {
    let kanjis = req
        .kanjis
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();

    let add_grid = std::cmp::max(0, req.extra_grid);
    let add_blank = std::cmp::max(0, req.extra_blank);

    // let upper = std::cmp::min(20, kanjis.len());
    let (pages, skipped_kanji) = create_pages(&kanjis, add_blank, add_grid, data.clone());
    fs::create_dir_all(&format!("out/{timestamp}")).unwrap();

    if req.pdf {
        create_pdf(&pages, &kanjis, timestamp);
    }
    if req.png {
        for (i, page) in pages.imgs.iter().enumerate() {
            let path = PathBuf::from(format!("out/{timestamp}/image-{i}.png"));
            page.save(&path).unwrap();
        }
    }

    let _ = open::that_detached(&format!("out/{timestamp}"));

    skipped_kanji.into_iter().collect::<String>()
}

impl Worker for AsyncHandler {
    type Init = ();
    type Input = AsyncHandlerMsg;
    type Output = AppMsg;

    fn init(_init: Self::Init, _sender: ComponentSender<Self>) -> Self {
        Self
    }

    fn update(&mut self, msg: AsyncHandlerMsg, sender: ComponentSender<Self>) {
        match msg {
            AsyncHandlerMsg::Start(k) => {
                sender
                    .output(AppMsg::SendMessage(String::from("Starting..."), true))
                    .unwrap();
                //TODO move this elsewhere later
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let _stroke_map = do_csv();
                let svgs = do_svgs();
                let data = Globals { _stroke_map, svgs };
                let skipped = process(k, data, timestamp);
                sender
                    .output(AppMsg::SendMessage(String::from("Completed\n"), true))
                    .unwrap();
                sender.output(AppMsg::End).unwrap();
                if !skipped.is_empty() {
                    sender
                        .output(AppMsg::SendMessage(
                            String::from("Skipped: {skipped}\n"),
                            true,
                        ))
                        .unwrap();
                }
            }
        }
    }
}
