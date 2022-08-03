use std::io::{Cursor, Read, Write};

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{
    http::header::ContentType, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use clap::{Parser, Subcommand};
use image::ImageOutputFormat;
use kanji_practice_sheet::{create_pages, find_free_port, pages::Pages, pdf_creation::create_pdf};
use serde::Deserialize;
use zip::{write::FileOptions, CompressionMethod};

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
    /// Start the webserver and browser
    ServerLocal {},
    /// Use the generator directly
    #[clap(arg_required_else_help = true)]
    Cli {
        //TODO interactive mode?
        kanjis: String,
        #[clap(short, long)]
        pdf: bool,
        #[clap(short, long, default_value_t = true)]
        files: bool,
        #[clap(short, long, default_value_t = 0)]
        blank: u16,
        #[clap(short, long, default_value_t = 0)]
        grid: u16,
    },
}

#[derive(Debug, Deserialize)]
struct KanjiRequest {
    kanjis: String,
    extra_grid: u16,
    extra_blank: u16,
    pdf: bool,
    png: bool,
    opt_space: bool,
    _coloring: Option<String>, // TODO can serde do this as enum?
}

async fn compress(pages: &Pages, pdf: bool, png: bool, kanjis: &str) -> std::io::Result<Vec<u8>> {
    let buf = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buf);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o755);

    if png {
        // TODO skip the last, check if last is blank, save if not
        let mut b = Vec::with_capacity(505728);
        for (i, page) in pages.imgs.iter().enumerate() {
            page.write_to(&mut b, ImageOutputFormat::Png).unwrap();
            zip.start_file(format!("page-{i}.png"), options).unwrap();
            zip.write_all(&b)?;
            b.clear();
        }
    }

    // PDF won't let us write to bufferrr
    if pdf {
        create_pdf(pages, kanjis);
        zip.start_file("result.pdf", options)?;
        let mut b = Vec::with_capacity(20000);
        let mut file = std::fs::File::open(&format!("out/{kanjis}.pdf")).unwrap();
        file.read_to_end(&mut b)?;
        zip.write_all(&b)?;
    }

    let writer = zip.finish()?;
    Ok(writer.into_inner())
}

async fn process_web(_: HttpRequest, req: web::Json<KanjiRequest>) -> impl Responder {
    let kanjis = req
        .kanjis
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();

    let add_grid = std::cmp::max(0, req.extra_grid);
    let add_blank = std::cmp::max(0, req.extra_blank);

    // Pick the 20 first kanjis or the whole thing
    let upper = std::cmp::min(20, kanjis.len());
    let kanjis = &kanjis.chars().take(upper).collect::<String>();
    let time = std::time::Instant::now();
    let (pages, skipped_kanji) = create_pages(kanjis, add_blank, add_grid);
    println!("processed in {:?}", time.elapsed());
    let (content_type, data) = if req.pdf && !req.png {
        create_pdf(&pages, kanjis);
        let mut b = Vec::with_capacity(20000);
        let mut file = std::fs::File::open(&format!("out/{}.pdf", kanjis)).unwrap();
        file.read_to_end(&mut b).unwrap();
        ("application/pdf", b)
    } else {
        (
            "application/zip",
            compress(&pages, req.pdf, req.png, kanjis).await.unwrap(),
        )
    };

    println!("zip in {:?}", time.elapsed());

    HttpResponse::Ok()
        .insert_header(("Content-Type", content_type))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .insert_header((
            "X-Skipped-Kanji",
            skipped_kanji.into_iter().collect::<String>(),
        ))
        .body(data)
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
async fn main() -> std::result::Result<(), std::io::Error> {
    let args = Args::parse();
    match args.command {
        Commands::ServerLocal {} => {
            let port = find_free_port().expect("Couldn't find any port to bind to.");
            println!("Binding to 127.0.0.1:{port}");
            let url = format!("127.0.0.1:{port}");

            let server_future = HttpServer::new(|| {
                App::new()
                    .route("/api/process/", web::post().to(process_web))
                    .route("/", web::get().to(homepage))
                    .service(Files::new("/assets", "./assets/"))
            })
            .bind(&url)?
            .run();
            let (result, _) =
                tokio::join!(server_future, kanji_practice_sheet::launch_browser(&url));
            result
        }
        Commands::Cli {
            kanjis,
            pdf,
            files,
            blank,
            grid,
        } => {
            let (pages, skipped_kanji) = create_pages(&kanjis, blank, grid);
            println!(
                "The following kanji were skipped, as they were not found in the KanjiVG Database: \n {:?}",
                skipped_kanji
            );

            if files {
                pages.save_pages(&kanjis);
            }
            if pdf {
                create_pdf(&pages, &kanjis);
            }
            Ok(())
        }
        Commands::Server {} => {
            let port = find_free_port().expect("Couldn't find any port to bind to.");
            println!("Binding to 127.0.0.1:{port}");
            let url = format!("127.0.0.1:{port}");

            HttpServer::new(|| {
                App::new()
                    .route("/api/process/", web::post().to(process_web))
                    .route("/", web::get().to(homepage))
                    .service(Files::new("/assets", "./assets/"))
            })
            .bind(&url)?
            .run()
            .await
        }
    }
}
