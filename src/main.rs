use actix_files::Files;
use actix_web::{
    http::header::ContentType, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use clap::{Parser, Subcommand};
use kanji_practice_sheet::{
    arg_parsing::kanji_to_filename,
    pages::Pages,
    pdf_creation::{create_pdf, kanji_to_png},
};
use serde::Deserialize;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the webserver
    Server {},
    /// Use the generator directly
    #[clap(arg_required_else_help = true)]
    Cli {
        //TODO interactive mode?
        kanjis: String,
        #[clap(short, long)]
        pdf: bool,
        #[clap(short, long, default_value_t = true)]
        files: bool,
    },
}

fn process(kanjis: &str, pdf: bool, files: bool) {
    let mut pages = Pages::default();
    pages.add_page();

    for kanji in kanjis.chars() {
        kanji_to_png(&mut pages, &kanji_to_filename(kanji));
    }
    if files {
        pages.save_pages(kanjis);
    }
    if pdf {
        create_pdf(&pages, kanjis);
    }
}

#[derive(Debug, Deserialize)]
struct Test {
    kanjis: String,
}

async fn process_web(_: HttpRequest, test: web::Json<Test>) -> impl Responder {
    dbg!(&test);
    let time = std::time::Instant::now();
    process(&test.kanjis, false, true);
    let time = time.elapsed();
    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body(format!("processed in {:?}", time))
}

async fn homepage(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok().content_type(ContentType::html()).body(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Kanji Sheet Generator</title>
</head>
<script src="assets/js/js.js"></script>
<body>
<form id="kanjis" action="">
  <div class="form-example">
    <label for="email">Enter your email: </label>
    <input type="text" name="kanjis" id="kanjis" required>
  </div>
  <div class="form-example">
    <input type="submit" value="Generate">
  </div>
</form>
</body>
</html>"#
            .to_string(),
    )
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::Server {} => {
            HttpServer::new(|| {
                App::new()
                    .route("/api/process/", web::post().to(process_web))
                    .route("/", web::get().to(homepage))
                    .service(Files::new("/assets", "./assets/"))
            })
            .bind("127.0.0.1:8000")?
            .run()
            .await
        }
        Commands::Cli { kanjis, pdf, files } => {
            process(&kanjis, pdf, files);
            Ok(())
        }
    }
}
