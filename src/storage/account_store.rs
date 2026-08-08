use crate::domain::account::Account;
use crate::domain::error::{GitNestError, Result};
use crate::storage::json_store::{read_json_file, write_json_file_atomic};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub trait AccountRepository: Send + Sync {
    fn add(&mut self, account: Account) -> Result<()>;
    fn remove(&mut self, id_or_username: &str) -> Result<Account>;
    fn find_by_id(&self, id: &str) -> Result<Option<Account>>;
    fn find_by_username(&self, username: &str) -> Result<Option<Account>>;
    fn list_all(&self) -> Result<Vec<Account>>;
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountsWrapper {
    accounts: Vec<Account>,
}

pub fn load_accounts(path: &Path) -> Result<Vec<Account>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let wrapper: AccountsWrapper = read_json_file(path)?;
    Ok(wrapper.accounts)
}

pub fn save_accounts(path: &Path, accounts: &[Account]) -> Result<()> {
    let wrapper = AccountsWrapper {
        accounts: accounts.to_vec(),
    };
    write_json_file_atomic(path, &wrapper)
}

pub struct JsonAccountRepository {
    storage_path: PathBuf,
}

impl JsonAccountRepository {
    pub fn new(storage_path: PathBuf) -> Self {
        Self { storage_path }
    }
}

impl AccountRepository for JsonAccountRepository {
    fn add(&mut self, account: Account) -> Result<()> {
        let mut accounts = load_accounts(&self.storage_path)?;
        if accounts
            .iter()
            .any(|a| a.github_username == account.github_username)
        {
            return Err(GitNestError::AccountAlreadyExists(
                account.github_username.clone(),
            ));
        }
        accounts.push(account);
        save_accounts(&self.storage_path, &accounts)
    }

    fn remove(&mut self, id_or_username: &str) -> Result<Account> {
        let mut accounts = load_accounts(&self.storage_path)?;
        let idx = accounts
            .iter()
            .position(|a| a.id == id_or_username || a.github_username == id_or_username)
            .ok_or_else(|| GitNestError::AccountNotFound(id_or_username.to_string()))?;

        let removed = accounts.remove(idx);
        save_accounts(&self.storage_path, &accounts)?;
        Ok(removed)
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Account>> {
        let accounts = load_accounts(&self.storage_path)?;
        Ok(accounts.into_iter().find(|a| a.id == id))
    }

    fn find_by_username(&self, username: &str) -> Result<Option<Account>> {
        let accounts = load_accounts(&self.storage_path)?;
        Ok(accounts.into_iter().find(|a| a.github_username == username))
    }

    fn list_all(&self) -> Result<Vec<Account>> {
        load_accounts(&self.storage_path)
    }
}
