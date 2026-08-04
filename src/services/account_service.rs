use crate::domain::account::Account;
use crate::domain::error::Result;
use crate::storage::account_store::AccountRepository;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AccountService {
    repo: Arc<Mutex<dyn AccountRepository>>,
}

impl AccountService {
    pub fn new(repo: Arc<Mutex<dyn AccountRepository>>) -> Self {
        Self { repo }
    }

    pub async fn add_account(&self, account: Account) -> Result<()> {
        let mut guard = self.repo.lock().await;
        guard.add(account)
    }

    pub async fn remove_account(&self, id_or_username: &str) -> Result<Account> {
        let mut guard = self.repo.lock().await;
        guard.remove(id_or_username)
    }

    pub async fn list_accounts(&self) -> Result<Vec<Account>> {
        let guard = self.repo.lock().await;
        guard.list_all()
    }

    pub async fn find_account(&self, id_or_username: &str) -> Result<Option<Account>> {
        let guard = self.repo.lock().await;
        if let Some(acc) = guard.find_by_id(id_or_username)? {
            return Ok(Some(acc));
        }
        guard.find_by_username(id_or_username)
    }
}
