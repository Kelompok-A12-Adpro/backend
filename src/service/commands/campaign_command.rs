#[derive(Debug)]
pub struct CreateCampaignCommand {
    pub user_id: i32,
    pub name: String,
    pub description: String,
    pub target_amount: f64,
}

#[derive(Debug)]
pub struct UpdateCampaignCommand {
    pub campaign_id: i32,
    pub user_id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub target_amount: Option<f64>,
}

#[derive(Debug)]
pub struct DeleteCampaignCommand {
    pub campaign_id: i32,
    pub user_id: i32,
}

#[derive(Debug)]
pub struct UploadEvidenceCommand {
    pub campaign_id: i32,
    pub user_id: i32,
    pub evidence_url: String,
}