use chrono::NaiveDate;
use mysql_async::prelude::FromRow;
use mysql_async::{Row, Value};
use mysql_common::bigdecimal::Zero;
use mysql_common::FromValueError;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct SalesByColorRanked {
    pub brew_color: String,
    pub beer_style: String,
    pub location: String,
    pub total_sales: f64,
    pub ranked: u32,
}

impl FromRow for SalesByColorRanked {
    fn from_row(row: Row) -> Self {
        let (brew_color, beer_style, location, total_sales, ranked): (String, String, String, f64, u32) =
            mysql_async::from_row(row);

        SalesByColorRanked {brew_color, beer_style, location, total_sales, ranked}
    }

    fn from_row_opt(row: Row) -> Result<Self, mysql_async::FromRowError> {
        let (brew_color, beer_style, location, total_sales, ranked): (String, String, String, f64, u32) =
            mysql_async::from_row(row);

        Ok(SalesByColorRanked {brew_color, beer_style, location, total_sales, ranked})
    }
}

impl SalesByColorRanked {

    pub fn clean_string_fields(&mut self){

        let clean_string_fields = |s: &str| s.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>();

        if self.brew_color.is_empty()  || self.brew_color == "null" {
            self.brew_color = String::from("Unknown")
        } else {
            self.brew_color = clean_string_fields(&self.brew_color)
        }

        if self.beer_style.is_empty()  || self.beer_style == "null" {
            self.beer_style = String::from("Unknown")
        } else {
            self.beer_style = clean_string_fields(&self.beer_style)
        }

        if self.location.is_empty()  || self.location == "null" {
            self.location = String::from("Unknown")
        } else {
            self.location = clean_string_fields(&self.location)
        }

        
    }

    pub fn clean_f64_fields(&mut self) {
        
        if self.total_sales == std::f64::MAX{
            self.total_sales = 0.0;
        }
    
        else if self.total_sales.is_nan() {
            self.total_sales = 0.0;
        }

        else if self.total_sales.is_zero() {
            self.total_sales= 0.0;
        }

        else if self.total_sales.is_finite() {
            //floting number is ok
        }
    }

    pub fn clean_u32_fields(&mut self) -> Result<(), &'static str> {

        if self.ranked == std::u32::MAX {
            self.ranked = 0;
        } 
        
        else if self.ranked.is_zero() {
            return Err("Invalid value for ranked: 0");
        }

        Ok(())
    }
        
}

#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct SalesByColorParams {
    pub date_from: NaiveDate,
    pub date_to: NaiveDate,
}

impl TryFrom<Value> for SalesByColorParams {
    type Error = FromValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        // Closure to create NaiveDate object from year, month, and day
        let from_ymd_opt = |y, m, d| NaiveDate::from_ymd_opt(y, m, d);

        // Match the Value variant to extract the year, month, and day components
        match value {
            Value::Date(year, month, day, _, _, _, _) => {
                // Use from_ymd_opt closure to create NaiveDate objects
                let date_from = match from_ymd_opt(year as i32, month as u32, day as u32) {
                    Some(date) => date,
                    None => return Err(FromValueError(value)),
                };

                let date_to = date_from; // Assuming date_to is the same as date_from

                Ok(SalesByColorParams { date_from, date_to })
            }
            _ => Err(FromValueError(value)),
        }
    }
}
