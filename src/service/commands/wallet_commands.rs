#[derive(Debug)]
pub struct TopUpWalletCommand {
    pub user_id: i32,
    pub method: String,
    pub phone_number: String,
    pub amount: f64,
}

#[derive(Debug)]
pub struct DeleteTopUpTransactionCommand {
    pub user_id: i32,
    pub transaction_id: i32,
}
