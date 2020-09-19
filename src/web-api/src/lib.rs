use util;

use actix_files as fs;
use actix_web::{App, HttpServer};

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    let mut app_dir = util::env::get_cwd().unwrap();
    app_dir = app_dir.join("app");

    HttpServer::new(move || {
        App::new().service(fs::Files::new("/app", app_dir.to_str().unwrap()).show_files_listing())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}