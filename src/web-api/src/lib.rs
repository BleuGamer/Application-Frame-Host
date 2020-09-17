use util;

use actix_files::NamedFile;
use actix_web::{HttpRequest, Result};
use std::path::PathBuf;

async fn index(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    Ok(NamedFile::open(path)?)
}

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    use actix_web::{web, App, HttpServer};

    let file = util::env::get_cwd().unwrap().join("App").join("index.html");
    println!("PATH: {}: ", file.display());

    HttpServer::new(move || App::new().route(file.to_str().unwrap(), web::get().to(index)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
