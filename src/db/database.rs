use std::env;
use dotenv::dotenv;

use mysql_async::{params, prelude::Queryable};

use crate::models::brewery::{SalesByColorRanked, SalesByColorParams};

#[derive(Clone)]
pub struct Database {
    pub pool: mysql_async::Pool,
}

impl Database {
    pub async fn init() -> Result<Self, mysql_async::Error> {
        dotenv().ok();
        let db_url = env::var("MYSQL_DB_URL").expect("MY SQL not det in .env file");
        let pool = mysql_async::Pool::new(db_url.as_str());
        Ok(Database { pool })
    }

    pub async fn sales_by_color_ranked(
        &self,
        date_params: SalesByColorParams,
    ) -> Result<Vec<SalesByColorRanked>, mysql_async::Error> {
        let date_from = &date_params.date_from;
        let date_to = &date_params.date_to;

        let query = r#"
                    WITH ranked_sales AS (
                        SELECT 
                            c.color AS brew_color, 
                            bd.beer_style, 
                            bd.location, 
                            SUM(bd.total_sales) AS total_sales,
                            ROW_NUMBER() OVER (PARTITION BY c.color, bd.beer_style ORDER BY SUM(bd.total_sales) DESC) AS ranked
                        FROM 
                            brewery_data bd
                        JOIN 
                            colors c ON bd.color = c.color_number
                        WHERE 
                            bd.brew_date >= :date_from AND bd.brew_date <= :date_to
                        GROUP BY 
                            c.color, bd.beer_style, bd.location
                    )
                    SELECT 
                        brew_color, 
                        beer_style, 
                        location, 
                        total_sales,
                        ranked
                    FROM 
                        ranked_sales
                    ORDER BY 
                        brew_color, beer_style, total_sales DESC;
                "#;

        let params = params! {
            "date_from" => date_from,
            "date_to" => date_to,
        };

        let mut conn = self.pool.get_conn().await?;
        let brew_color_sales: Vec<SalesByColorRanked> = conn.exec(query, params).await.expect("Failed to retrieve data for Sales by color Ranked.");

        //Cleaning the data
        let mut cleaned_sales: Vec<SalesByColorRanked> = Vec::new();
        for mut sales_record in brew_color_sales{
            sales_record.clean_string_fields();
            sales_record.clean_f64_fields();

            match sales_record.clean_u32_fields(){
                Ok(_)=> cleaned_sales.push(sales_record),
                Err(err)=>{
                    eprint!("Error cleaning u32 field: {}", err)
                }
            }  
        }


        Ok(cleaned_sales)
    }


   
}


#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use super::*;

    async fn setup_test_database() -> Database {
        dotenv().ok();
        let db_url = env::var("MYSQL_DB_URL").expect("MYSQL_DB_URL not set in .env file");
        let pool = mysql_async::Pool::new(db_url.as_str());
        Database { pool }
    }

    #[tokio::test]
    #[allow(non_snake_case)] 
    async fn test_name() {
        // Arrange: Initialize the connection pool
        let db = setup_test_database().await;

        // Act: Call the function you want to test
        // Parse date strings into NaiveDate objects
        let date_from = NaiveDate::parse_from_str("2020-01-01", "%Y-%m-%d").expect("Failed to parse date_from string");
        let date_to = NaiveDate::parse_from_str("2020-03-01", "%Y-%m-%d").expect("Failed to parse date_to string");

        // Create SalesByColorParams object
        let dates = SalesByColorParams {
            date_from,
            date_to,
        };

        let result = db.sales_by_color_ranked(dates).await;

        // Assert: Check if the result is as expected
        match result {
            Ok(SalesByColor) => {
                assert!(!SalesByColor.is_empty());

                for color_sales in SalesByColor {
                    if !color_sales.brew_color.is_empty() {
                        // It's ok
                    } else {
                        panic!("Brew color is missing or empty.")
                    }

                    if !color_sales.beer_style.is_empty() {
                        // It's ok
                    } else {
                        panic!("Beer style is missing or empty.")
                    }

                    if !color_sales.location.is_empty() {
                        // It's ok
                    } else {
                        panic!("Location is missing or empty.")
                    }

                    if color_sales.total_sales.is_finite() {
                        // total_sales is a finite number (f64)
                    } else {
                        panic!("Total sales is missing or not a finite number");
                    }

                    assert!(color_sales.ranked >= 1, "Pcs should be a non-negative number");
                }
            }
            Err(e) => {
                // Handle error if necessary
                panic!("Error occurred: {:?}", e);
            }
        }
    }

    
}