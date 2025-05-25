#[derive(Debug)]
pub struct MakeDonationCommand {
    pub donor_id: i32,
    pub campaign_id: i32,
    pub amount: i64,
    pub message: Option<String>,
}

#[derive(Debug)]
pub struct DeleteDonationMessageCommand {
    pub donation_id: i32,
    pub user_id: i32,
}