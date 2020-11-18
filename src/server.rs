mod database;

#[macro_use]
extern crate actix_web;
#[macro_use]
extern crate log;

use std::env;
use actix_web::{web, HttpServer, App, HttpResponse};
use crate::database::{DataBase, Collector};
use actix_web::middleware::Logger;
use actix_web::http::StatusCode;

const DEFAULT_SERVER_ADDRESS: &str = "0.0.0.0:8000";

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let server_address = env::args().collect::<Vec<String>>().get(1)
        .map_or_else(|| String::from(DEFAULT_SERVER_ADDRESS), |arg| String::from(arg));
    let database = web::Data::new(DataBase::new(String::from("data/db.json")).await);

    HttpServer::new(move || {
        App::new()
            .app_data(database.clone())
            .wrap(Logger::default())
            .service(regi)
    })
        .bind(server_address)?
        .run()
        .await?;

    Ok(())
}

#[post("/")]
pub async fn regi(database: web::Data<DataBase>, req: web::Json<Collector>) -> actix_web::Result<HttpResponse> {
    let total = database.add(req.into_inner()).await;
    info!("Total {} trials", total);
    Ok(HttpResponse::build(StatusCode::OK).body(""))
}
