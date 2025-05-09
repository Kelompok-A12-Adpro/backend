use crate::errors::AppError;
use crate::strategy::payment::PaymentMethod;

pub struct Dana;

impl PaymentMethod for Dana {
    fn pay(&self, amount: f64, phone_number: &str) -> Result<(), AppError> {
        // Integration with DANA would go here
        println!("Processing DANA payment of {} to {}", amount, phone_number);
        Ok(())
    }
    
    fn get_name(&self) -> &'static str {
        "DANA"
    }
}
