use crate::db::database::Database;
use actix_web::web::Data;
use actix_web::{HttpResponse, Responder,web,get};



use crate::models::brewery::SalesByColorParams;
use validator::Validate; 

#[get("/sales_by_color_ranked")]
async fn sales_by_color_ranked(
    db: Data<Database>,
    date_params: web::Query<SalesByColorParams>,
) -> impl Responder {

    let date_params = date_params.into_inner();

    // Validate the parameters
    if let Err(validation_errors) = date_params.validate() {
        // Return validation errors if any
        return HttpResponse::BadRequest().json(validation_errors);
    }

    match db.sales_by_color_ranked(date_params).await {
        Ok(found_orders) => {
            if found_orders.is_empty() {
                HttpResponse::NotFound().body("No data available in the database")
            } else {
                HttpResponse::Ok().json(found_orders)
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Error retrieving Color Sales"),
    }
}
