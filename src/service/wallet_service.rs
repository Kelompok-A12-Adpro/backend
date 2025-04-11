use crate::errors::AppError;
use crate::model::wallet::Wallet;
use crate::model::wallet_transaction::{TransactionType, WalletTransaction};
use crate::repository::wallet_repo::WalletRepository;
use crate::repository::wallet_transaction_repo::WalletTransactionRepository;
use crate::service::commands::wallet_commands::{TopUpWalletCommand, DeleteTopUpTransactionCommand};

use crate::strategy::payment::{PaymentMethod, Gopay, Dana};
use std::sync::Arc;

pub struct WalletService {
    wallet_repo: Arc<dyn WalletRepository>,
    transaction_repo: Arc<dyn WalletTransactionRepository>,
}

impl WalletService {
    pub fn new(
        wallet_repo: Arc<dyn WalletRepository>,
        transaction_repo: Arc<dyn WalletTransactionRepository>,
    ) -> Self {
        WalletService {
            wallet_repo,
            transaction_repo,
        }
    }

    pub async fn top_up(&self, cmd: TopUpWalletCommand) -> Result<WalletTransaction, AppError> {
        if cmd.amount <= 0.0 {
            return Err(AppError::ValidationError("Amount must be positive".into()));
        }

        let strategy: Box<dyn PaymentMethod> = match cmd.method.to_lowercase().as_str() {
            "gopay" => Box::new(Gopay),
            "dana" => Box::new(Dana),
            _ => return Err(AppError::ValidationError("Invalid payment method".into())),
        };

        // Bisa tambahkan validasi format phone_number, dll.

        // Simulasikan transaksi berhasil
        strategy.pay(cmd.amount); // kalau nanti mau connect ke payment gateway

        let mut wallet = self
            .wallet_repo
            .find_by_user_id(cmd.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Wallet not found".into()))?;

        wallet.balance += cmd.amount;

        self.wallet_repo.update_balance(wallet.user_id, wallet.balance).await?;

        // Simpan transaksi
        let tx = self
            .transaction_repo
            .create_transaction(wallet.user_id, TransactionType::TopUp, cmd.amount)
            .await?;

        Ok(tx)
    }

    pub async fn get_wallet(&self, user_id: i32) -> Result<Wallet, AppError> {
        self.wallet_repo
            .find_by_user_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Wallet not found".into()))
    }

    pub async fn delete_topup_transaction(
        &self,
        cmd: DeleteTopUpTransactionCommand,
    ) -> Result<(), AppError> {
        let tx = self
            .transaction_repo
            .find_by_id(cmd.transaction_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Transaction not found".into()))?;

        if tx.user_id != cmd.user_id {
            return Err(AppError::Forbidden("You cannot delete this transaction".into()));
        }

        if tx.transaction_type != TransactionType::TopUp {
            return Err(AppError::ValidationError("Only top-up transactions can be deleted".into()));
        }

        self.transaction_repo.delete_transaction(cmd.transaction_id).await?;

        Ok(())
    }

    pub async fn get_transaction_history(&self, user_id: i32) -> Result<Vec<WalletTransaction>, AppError> {
        self.transaction_repo.find_by_user_id(user_id).await
    }
}
