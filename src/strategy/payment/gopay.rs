use crate::errors::AppError;
use crate::strategy::payment::PaymentMethod;

pub struct Gopay;

impl PaymentMethod for Gopay {
    fn pay(&self, amount: f64, phone_number: &str) -> Result<(), AppError> {
        // Integration with GOPAY would go here
        println!("Processing GOPAY payment of {} to {}", amount, phone_number);
        Ok(())
    }
    
    fn get_name(&self) -> &'static str {
        "GOPAY"
    }
}
