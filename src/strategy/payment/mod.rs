use crate::errors::AppError;

pub mod dana;
pub mod gopay;

pub trait PaymentMethod {
    fn pay(&self, amount: f64, phone_number: &str) -> Result<(), AppError>;
    fn get_name(&self) -> &'static str;
}
