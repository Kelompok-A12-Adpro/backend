use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataStatistic {
    pub active_campaigns_count: i32,
    pub total_donations_amount: i64,
    pub daily_transaction_count: i32,
    pub weekly_transaction_count: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransactionData {
    pub name: String,
    pub transactions: i32,
    pub amount: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecentDonation {
    pub id: i32,
    pub amount: i64,
    pub campaign: String,
    pub date: String,
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_data_statistic_creation_and_fields() {
        let stats = DataStatistic {
            active_campaigns_count: 5,
            total_donations_amount: 10000,
            daily_transaction_count: 50,
            weekly_transaction_count: 300,
        };

        assert_eq!(stats.active_campaigns_count, 5);
        assert_eq!(stats.total_donations_amount, 10000);
        assert_eq!(stats.daily_transaction_count, 50);
        assert_eq!(stats.weekly_transaction_count, 300);
    }

    #[test]
    fn test_data_statistic_serialization() {
        let stats = DataStatistic {
            active_campaigns_count: 1,
            total_donations_amount: 5000,
            daily_transaction_count: 10,
            weekly_transaction_count: 70,
        };

        let serialized = serde_json::to_string(&stats).unwrap();
        let deserialized: DataStatistic = serde_json::from_str(&serialized).unwrap();

        assert_eq!(
            stats.active_campaigns_count,
            deserialized.active_campaigns_count
        );
        assert_eq!(
            stats.total_donations_amount,
            deserialized.total_donations_amount
        );
    }

    #[test]
    fn test_data_statistic_clone_and_debug() {
        let stats = DataStatistic {
            active_campaigns_count: 3,
            total_donations_amount: 7500,
            daily_transaction_count: 25,
            weekly_transaction_count: 175,
        };

        let cloned = stats.clone();
        assert_eq!(stats.active_campaigns_count, cloned.active_campaigns_count);

        let debug_output = format!("{:?}", stats);
        assert!(debug_output.contains("DataStatistic"));
    }

    #[test]
    fn test_transaction_data_creation_and_fields() {
        let transaction = TransactionData {
            name: "Test Campaign".to_string(),
            transactions: 100,
            amount: 50000,
        };

        assert_eq!(transaction.name, "Test Campaign");
        assert_eq!(transaction.transactions, 100);
        assert_eq!(transaction.amount, 50000);
    }

    #[test]
    fn test_transaction_data_partial_eq() {
        let transaction1 = TransactionData {
            name: "Campaign A".to_string(),
            transactions: 50,
            amount: 25000,
        };

        let transaction2 = TransactionData {
            name: "Campaign A".to_string(),
            transactions: 50,
            amount: 25000,
        };

        let transaction3 = TransactionData {
            name: "Campaign B".to_string(),
            transactions: 50,
            amount: 25000,
        };

        assert_eq!(transaction1, transaction2);
        assert_ne!(transaction1, transaction3);
    }

    #[test]
    fn test_transaction_data_clone_and_debug() {
        let transaction = TransactionData {
            name: "Clone Test".to_string(),
            transactions: 75,
            amount: 37500,
        };

        let cloned = transaction.clone();
        assert_eq!(transaction, cloned);

        let debug_output = format!("{:?}", transaction);
        assert!(debug_output.contains("TransactionData"));
        assert!(debug_output.contains("Clone Test"));
    }
}
