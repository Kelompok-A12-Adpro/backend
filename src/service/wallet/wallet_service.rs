use std::sync::Arc;
use chrono::Utc;
use crate::errors::AppError;
use crate::model::wallet::wallet::Wallet;
use crate::model::wallet::transaction::Transaction;
use crate::repository::wallet::wallet_repository::WalletRepository;
use crate::repository::wallet::transaction_repository::TransactionRepository;
use crate::strategy::payment::{PaymentMethod, self};
use crate::strategy::payment::gopay::Gopay;
use crate::strategy::payment::dana::Dana;

pub struct WalletService {
    wallet_repo: Arc<dyn WalletRepository>,
    transaction_repo: Arc<dyn TransactionRepository>,
}

impl WalletService {
    pub fn new(
        wallet_repo: Arc<dyn WalletRepository>, 
        transaction_repo: Arc<dyn TransactionRepository>
    ) -> Self {
        WalletService { 
            wallet_repo,
            transaction_repo,
        }
    }

    pub async fn get_wallet(&self, user_id: i32) -> Result<Wallet, AppError> {
        // Check if wallet exists for user
        let wallet_opt = self.wallet_repo.find_by_user_id(user_id).await?;
        
        // Create wallet if it doesn't exist
        match wallet_opt {
            Some(wallet) => Ok(wallet),
            None => {
                // Create new wallet directly (not inside a closure)
                let new_wallet = self.wallet_repo.create_wallet_if_not_exists(user_id).await?;
                Ok(new_wallet)
            }
        }
    }

    pub async fn top_up(
        &self,
        user_id: i32,
        method: &str,
        phone_number: &str,
        amount: f64,
    ) -> Result<Wallet, AppError> {
        if amount <= 0.0 {
            return Err(AppError::ValidationError("Amount must be positive".into()));
        }

        let strategy: Box<dyn PaymentMethod> = match method.to_uppercase().as_str() {
            "GOPAY" => Box::new(Gopay),
            "DANA" => Box::new(Dana),
            _ => return Err(AppError::ValidationError("Invalid payment method. Use GOPAY or DANA".into())),
        };

        // Process payment through the payment gateway
        strategy.pay(amount, phone_number)?;

        // Get or create wallet
        let mut wallet = self.get_wallet(user_id).await?;
        
        // Update balance
        wallet.balance += amount;
        self.wallet_repo.update_balance(user_id, wallet.balance).await?;
        
        // Record transaction
        let transaction = Transaction {
            id: 0, // Will be set by the database
            wallet_id: wallet.id,
            transaction_type: "topup".to_string(),
            amount,
            method: Some(strategy.get_name().to_string()),
            phone_number: Some(phone_number.to_string()),
            campaign_id: None,
            created_at: Utc::now().naive_utc(),
            is_deleted: false,
        };
        
        self.transaction_repo.create_transaction(transaction).await?;
        
        Ok(wallet)
    }
    
    pub async fn get_transactions(&self, user_id: i32) -> Result<Vec<Transaction>, AppError> {
        let wallet = self.get_wallet(user_id).await?;
        let transactions = self.transaction_repo.get_wallet_transactions(wallet.id).await?;
        Ok(transactions)
    }
    
    pub async fn delete_transaction(&self, user_id: i32, transaction_id: i32) -> Result<(), AppError> {
        let wallet = self.get_wallet(user_id).await?;
        
        // Get the transaction to check its type
        let transactions = self.transaction_repo.get_wallet_transactions(wallet.id).await?;
        let transaction = transactions.iter()
            .find(|t| t.id == transaction_id)
            .ok_or_else(|| AppError::NotFound("Transaction not found".into()))?;
        
        // Only allow deletion of topup transactions
        if transaction.transaction_type != "topup" {
            // Use a different AppError variant instead of PermissionDenied
            return Err(AppError::ValidationError("Only topup transactions can be deleted".into()));
        }
        
        self.transaction_repo.delete_transaction(transaction_id, wallet.id).await
    }
    
    pub async fn withdraw_campaign_funds(
        &self, 
        fundraiser_id: i32, 
        campaign_id: i32
    ) -> Result<f64, AppError> {
        // This would typically involve:
        // 1. Check if campaign is completed
        // 2. Check if user is the fundraiser
        // 3. Calculate total donations
        // 4. Transfer to fundraiser's wallet
        
        // For this implementation, we'll just get the donations and simulate the withdrawal
        let donations = self.transaction_repo.get_campaign_donations(campaign_id).await?;
        let total_amount: f64 = donations.iter()
            .filter(|d| d.transaction_type == "donation")
            .map(|d| d.amount)
            .sum();
            
        if total_amount <= 0.0 {
            return Err(AppError::ValidationError("No funds available for withdrawal".into()));
        }
        
        // Update fundraiser's wallet
        let mut wallet = self.get_wallet(fundraiser_id).await?;
        wallet.balance += total_amount;
        self.wallet_repo.update_balance(fundraiser_id, wallet.balance).await?;
        
        // Record withdrawal transaction
        let transaction = Transaction {
            id: 0,
            wallet_id: wallet.id,
            transaction_type: "withdrawal".to_string(),
            amount: total_amount,
            method: None,
            phone_number: None,
            campaign_id: Some(campaign_id),
            created_at: Utc::now().naive_utc(),
            is_deleted: false,
        };
        
        self.transaction_repo.create_transaction(transaction).await?;
        
        Ok(total_amount)
    }
}