use util;

use actix_files as fs;
use actix_web::{error, get, guard, web, App, HttpResponse, HttpServer, Result};

#[get("/")]
async fn index() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("./app/index.html")?)
}

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    let mut dir = util::env::get_cwd().unwrap();
    dir.push("app");

    HttpServer::new(move || {
        App::new()
            .service(fs::Files::new("/", dir.to_str().unwrap()).index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
