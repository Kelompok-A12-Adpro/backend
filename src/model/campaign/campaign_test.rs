#[cfg(test)]
mod tests {
    use crate::model::campaign::campaign::{Campaign, CampaignStatus};
    use chrono::Utc;
    use serde_json;

    fn fixed_utc_datetime() -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::parse_from_rfc3339("2023-10-26T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn create_test_campaign() -> Campaign {
        let now = fixed_utc_datetime();
        Campaign {
            id: 1,
            user_id: 10,
            name: String::from("Save the Oceans"),
            description: String::from("Campaign to clean oceans"),
            target_amount: 10000,
            collected_amount: 0,
            start_date: now,
            end_date: now + chrono::Duration::days(30),
            image_url: None,
            status: CampaignStatus::PendingVerification,
            created_at: now,
            updated_at: now,
            evidence_url: None,
            evidence_uploaded_at: None,
        }
    }

    #[test]
    fn test_campaign_creation() {
        let campaign = create_test_campaign();
        assert_eq!(campaign.id, 1);
        assert_eq!(campaign.user_id, 10);
        assert_eq!(campaign.name, "Save the Oceans");
        assert_eq!(campaign.target_amount, 10000);
        assert_eq!(campaign.status, CampaignStatus::PendingVerification);
    }

    #[test]
    fn test_campaign_serialization() {
        let campaign = create_test_campaign();
        let serialized = serde_json::to_string(&campaign).unwrap();
        assert!(serialized.contains("\"name\":\"Save the Oceans\""));
        assert!(serialized.contains("\"target_amount\":10000"));
    }

    #[test]
    fn test_campaign_status_equality() {
        assert_eq!(CampaignStatus::PendingVerification, CampaignStatus::PendingVerification);
        assert_ne!(CampaignStatus::PendingVerification, CampaignStatus::Active);
    }
}