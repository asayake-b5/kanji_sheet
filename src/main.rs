use kanji_practice_sheet::worker::{AppMsg, AsyncHandler, AsyncHandlerMsg, KanjiRequest};
use relm4::{
    gtk::{
        self,
        prelude::{
            BoxExt, ButtonExt, CheckButtonExt, EditableExt, EntryBufferExtManual, EntryExt,
            GtkWindowExt, OrientableExt, TextBufferExt, TextViewExt, WidgetExt,
        },
        Adjustment, EntryBuffer,
    },
    Component, ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SimpleComponent,
    WorkerController,
};

struct AppModel {
    sensitive: bool,
    show_button: bool,
    grid_lines: f64,
    empty_lines: f64,
    pdf_toggled: bool,
    png_toggled: bool,
    kanjis: EntryBuffer,
    worker: WorkerController<AsyncHandler>,
    buffer: gtk::TextBuffer,
}

#[derive(Debug)]
pub enum AppInMsg {
    End,
    PNGToggled,
    PDFToggled,
    Start,
    Recheck,
    UpdateGrid(f64),
    UpdateEmpty(f64),
    UpdateBuffer(String, bool),
}

#[derive(Debug)]
enum AppOutMsg {}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Input = AppInMsg;

    type Output = AppOutMsg;
    type Init = u8;

    // Initialize the UI.
    fn init(
        _: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppModel {
            sensitive: true,
            empty_lines: 0.0,
            grid_lines: 0.0,
            show_button: false,
            kanjis: EntryBuffer::new(Some("あいえ事漢字感じ")),
            buffer: gtk::TextBuffer::new(None),
            pdf_toggled: false,
            png_toggled: false,
            worker: AsyncHandler::builder().detach_worker(()).forward(
                sender.input_sender(),
                |msg| match msg {
                    AppMsg::End => AppInMsg::End,
                    AppMsg::SendMessage(s, b) => AppInMsg::UpdateBuffer(s, b),
                },
            ),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppInMsg::UpdateBuffer(msg, delete) => {
                if delete {
                    let (mut start, mut end) = self.buffer.bounds();
                    self.buffer.delete(&mut start, &mut end);
                }
                self.buffer.insert_at_cursor(&msg);
            }
            AppInMsg::Start => {
                self.sensitive = false;
                let k = KanjiRequest {
                    kanjis: self.kanjis.text().to_string(),
                    extra_grid: self.grid_lines.floor() as u16,
                    extra_blank: self.empty_lines.floor() as u16,
                    pdf: self.pdf_toggled,
                    png: self.png_toggled,
                    _opt_space: false, //TODO what even was this
                    _coloring: None,
                };
                self.worker
                    .sender()
                    .send(AsyncHandlerMsg::Start(k))
                    .unwrap();
            }
            AppInMsg::End => {
                self.sensitive = true;
            }
            AppInMsg::PDFToggled => self.pdf_toggled = !self.pdf_toggled,
            AppInMsg::PNGToggled => self.png_toggled = !self.png_toggled,
            AppInMsg::Recheck => {
                self.show_button =
                    self.kanjis.length() > 0 && (self.png_toggled || self.pdf_toggled)
            }
            AppInMsg::UpdateGrid(new_value) => self.grid_lines = new_value,
            AppInMsg::UpdateEmpty(new_value) => self.empty_lines = new_value,
        }
    }

    view! {
        gtk::Window {
            set_title: Some("Kanji Sheet Generator"),
            set_default_width: 600,
            set_default_height: 400,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Box {
                    #[watch]
                    set_sensitive: model.sensitive,

                    set_spacing: 5,
                    set_margin_all: 5,
                    set_orientation: gtk::Orientation::Horizontal,
                    gtk::Label {
                        set_label: "Kanjis to put in the sheet:"

                    },
                    gtk::Entry {
                        set_buffer: &model.kanjis,
                        connect_changed => AppInMsg::Recheck,

                    },
                },
                gtk::Box {
                    #[watch]
                    set_sensitive: model.sensitive,
                    set_spacing: 5,
                    set_margin_all: 5,
                    set_orientation: gtk::Orientation::Horizontal,
                    gtk::Label {
                        set_label: "Numbers of added lines:"
                    },
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    relm4::gtk::SpinButton::builder()
                    .adjustment(&Adjustment::new(0.0, 0.0, 50.0, 1.0, 0.0, 0.0))
                    .build(){
                        connect_value_changed[sender] => move |x| {
                            sender.input(AppInMsg::UpdateGrid(x.value()))
                    }},
                    gtk::Label {
                            set_label: "Grid lines"
                        }
                },
                    gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    relm4::gtk::SpinButton::builder()
                    .adjustment(&Adjustment::new(0.0, 0.0, 50.0, 1.0, 0.0, 0.0))
                    .build(){
                        connect_value_changed[sender] => move |x| {
                            sender.input(AppInMsg::UpdateEmpty(x.value()))
                    }},
                    gtk::Label {
                            set_label: "Empty square lines"
                        }
                }
                },

                gtk::Box {
                    #[watch]
                    set_sensitive: model.sensitive,
                    set_spacing: 5,
                    set_margin_all: 5,
                    set_orientation: gtk::Orientation::Horizontal,
                    gtk::Label {
                        set_label: "Export Formats:"
                    },
                    relm4::gtk::CheckButton::builder()
                        .label("PNG")
                        .build(){
                            connect_toggled[sender] => move |_| {
                                sender.input(AppInMsg::PNGToggled);
                                sender.input(AppInMsg::Recheck)
                            }
                    },
                    relm4::gtk::CheckButton::builder()
                        .label("PDF")
                        .build(){
                            connect_toggled[sender] => move |_| {
                                sender.input(AppInMsg::PDFToggled);
                                sender.input(AppInMsg::Recheck)
                            }
                    },
                },

                append = if model.show_button {
                    gtk::Button::with_label("Generate Sheets !") {
                        #[watch]
                        set_sensitive: model.sensitive,
                        connect_clicked[sender] => move |_| {
                            sender.input(AppInMsg::Start);
                        }
                }} else {
                    gtk::Label{
                        set_label: "Please fill all mandatory fields"
                    }
                },


                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 5,

                    gtk::ScrolledWindow {
                        set_min_content_height: 380,

                        #[wrap(Some)]
                        set_child = &gtk::TextView {
                            set_buffer: Some(&model.buffer),
                            set_editable: false,
                            // #[watch]
                            // set_visible: model.file_name.is_some(),
                        },
                    }},
                // else if model.show_indicator {
                //     gtk::Spinner {
                //         set_spinning: true,
                //     }
                // }

            }
        }
    }
}

fn main() -> std::result::Result<(), std::io::Error> {
    let app = RelmApp::new("asayake.kanji.sheet");
    app.run::<AppModel>(0);

    Ok(())

    // let args = Args::parse();
    // match args.command {
    //     Commands::Cli {
    //         kanjis,
    //         pdf,
    //         files,
    //         blank,
    //         grid,
    //     } => {
    //         // FIXME
    //         let (pages, skipped_kanji) = create_pages(&kanjis, blank, grid);
    //         println!(
    //             "The following kanji were skipped, as they were not found in the KanjiVG Database: \n {:?}",
    //             skipped_kanji
    //         );

    //         if files {
    //             pages.save_pages(&kanjis);
    //         }
    //         if pdf {
    //             create_pdf(&pages, &kanjis);
    //         }
    //         Ok(())
    //     }
    //     Commands::Server {} => {
    //         let port = find_free_port().expect("Couldn't find any port to bind to.");
    //         println!("Binding to 127.0.0.1:{port}");
    //         let url = format!("0.0.0.0:{port}");

    //         HttpServer::new(move || {
    //             let cors = Cors::permissive();
    //             App::new()
    //                 .app_data(Data::clone(&data))
    //                 .wrap(cors)
    //                 .route("/api/process/", web::post().to(process_web))
    //                 .route("/", web::get().to(homepage))
    //                 .service(Files::new("/assets", "./assets/"))
    //         })
    //         .bind(&url)?
    //         .run()
    //         .await
    //     }
    // }
}
