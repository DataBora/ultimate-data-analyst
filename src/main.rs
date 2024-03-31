use actix_web::web::Data;
use actix_web::{App, HttpServer};

mod db;
mod models;
mod api;

use crate::db::database::Database;

use api::mysqlapi::sales_by_color_ranked;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match Database::init().await {
        Ok(db) => {
            println!("Database initialized successfully");
            let db_data = Data::new(db);

            HttpServer::new(move || {
                App::new()
                    .app_data(db_data.clone())
                    .service(sales_by_color_ranked)
                
            })
            .bind("127.0.0.1:8080")?
            .run()
            .await
        }
        Err(err) => {
            eprintln!("Error connecting to the database: {}", err);
            std::process::exit(1);
        }
    }
}