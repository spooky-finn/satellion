use crate::{config::Chain, db, repository::TokenRepository};
use diesel::result;

pub struct TokenTracker {
    repository: TokenRepository,
}

impl TokenTracker {
    pub fn new(repo: &TokenRepository) -> Self {
        Self {
            repository: repo.clone(),
        }
    }

    pub fn insert_default_tokens(&self, tokens: Vec<db::Token>) -> Result<usize, result::Error> {
        return self.repository.insert_or_ignore_many(tokens);
    }

    pub fn load_all(
        &self,
        wallet_id: i32,
        chain_id: Chain,
    ) -> Result<Vec<db::Token>, result::Error> {
        return self.repository.load(wallet_id, chain_id);
    }
}
