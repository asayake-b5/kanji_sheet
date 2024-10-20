use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use relm4::{ComponentSender, Worker};

use crate::{
    create_pages, do_csv, do_svgs,
    pdf_creation::create_pdf,
    Globals,
    KanjiToPngErrors::{FileNotFound, LoadFontError, Undefined, UnlikelyError},
};

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

fn process(
    req: KanjiRequest,
    data: Globals,
    timestamp: u128,
    sender: &ComponentSender<AsyncHandler>,
) -> Result<String, ()> {
    let kanjis = req
        .kanjis
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();

    let add_grid = std::cmp::max(0, req.extra_grid);
    let add_blank = std::cmp::max(0, req.extra_blank);

    // let upper = std::cmp::min(20, kanjis.len());
    let Ok((pages, skipped_kanji)) = create_pages(&kanjis, add_blank, add_grid, data.clone())
    else {
        //TODO send message of aborting or something
        let _ = sender.output(AppMsg::SendMessage("Error creating pages, aborting. \n Please contact with the used kanjis for troubleshooting.".to_string(), false ));
        let _ = sender.output(AppMsg::End);
        return Err(());
    };
    let _ = fs::create_dir_all(&format!("out/{timestamp}"));

    if req.pdf {
        if let Err(e) = create_pdf(&pages, &kanjis, timestamp) {
            match e {
                LoadFontError => {
                    let _ = sender.output(AppMsg::SendMessage(
                        "Error loading the font for PDF creation.".to_string(),
                        false,
                    ));
                }
                Undefined => {
                    let _ = sender.output(AppMsg::SendMessage(
                        "Error rendering the pdf file.".to_string(),
                        false,
                    ));
                }
                UnlikelyError => {
                    let _ = sender.output(AppMsg::SendMessage(
                        "You shouldn't see this error message, please contact me with details of the kanjis used".to_string(),
                        false,
                    ));
                }
                FileNotFound => {}
            }
            if !req.png {
                return Err(());
            }
        };
    }
    if req.png {
        for (i, page) in pages.imgs.iter().enumerate() {
            let path = PathBuf::from(format!("out/{timestamp}/image-{i}.png"));
            let _ = page.save(&path);
        }
    }

    let _ = open::that_detached(&format!("out/{timestamp}"));

    Ok(skipped_kanji.into_iter().collect::<String>())
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
                let _ = sender.output(AppMsg::SendMessage(String::from("Starting..."), true));
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::new(00000, 0))
                    .as_millis();
                //TODO move this elsewhere later
                let _stroke_map = do_csv();
                let svgs = do_svgs();
                let data = Globals { _stroke_map, svgs };
                if let Ok(skipped) = process(k, data, timestamp, &sender) {
                    let _ = sender.output(AppMsg::SendMessage(String::from("Completed\n"), true));
                    let _ = sender.output(AppMsg::End);
                    if !skipped.is_empty() {
                        let _ = sender.output(AppMsg::SendMessage(
                            String::from("Skipped: {skipped}\n"),
                            true,
                        ));
                    }
                }
            }
        }
    }
}
