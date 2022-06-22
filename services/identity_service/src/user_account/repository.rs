use std::collections::HashSet;
use std::error::Error;
use std::ops::Add;

use async_trait::async_trait;
use common_macros::hash_set;
use thiserror::Error;
use uuid::Uuid;

use super::types::AccountAttr;
use super::UserAccount;


#[derive(Debug, Error)]
pub enum CreateAccountError {
    #[error("An account with the given email address already exists.")]
    DuplicateAccount,

    #[error("{0}")]
    Validation(&'static str),

    #[error(transparent)]
    Other(#[from] Box<dyn Error>),
}

#[derive(Debug, Error)]
pub enum GetAccountError {
    #[error("Account not found.")]
    NotFound,

    #[error(transparent)]
    Serde(serde_ddb::Error),

    #[error(transparent)]
    Other(#[from] Box<dyn Error>),
}


#[derive(Clone, Debug)]
pub enum AccountLookup {
    ById(Uuid),
    ByEmail(String),
}

#[derive(Clone, Debug)]
pub enum AccountAttributes {
    Profile,
    Permissions,
    Password,
    Specific(Vec<AccountAttr>),
}


#[async_trait]
pub trait AccountsRepository {
    async fn create_account<'a>(&self, account: &'a UserAccount) -> Result<&'a Uuid, CreateAccountError>;

    async fn get_account(
        &self,
        lookup: &AccountLookup,
        attrs: &AccountAttributes,
    ) -> Result<UserAccount, GetAccountError>;
}


impl AccountAttributes {
    pub fn fields(&self) -> HashSet<AccountAttr> {
        use AccountAttr::*;

        match self {
            Self::Password => hash_set! { Password },
            Self::Profile => hash_set! { AccountId, Email, FirstName, LastName, Discoverable, AccountState },
            Self::Permissions => hash_set! { PermissionsDocument },
            Self::Specific(attrs) => attrs.iter().copied().collect(),
        }
    }

    pub fn ddb_projection_expression(&self) -> String {
        let mut buf = String::new();
        for it in self.fields() {
            if !buf.is_empty() {
                buf.push(',');
            }
            buf.push_str(&it.to_string());
        }
        buf
    }
}

impl Add for AccountAttributes {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut fields = self.fields();
        fields.extend(rhs.fields().iter());
        Self::Specific(fields.into_iter().collect())
    }
}


#[cfg(test)]
mod test_account_attributes {
    use crate::user_account::repository::AccountAttributes;
    use crate::user_account::types::AccountAttr;

    #[test]
    fn add() {
        let a = AccountAttributes::Permissions;
        let b = AccountAttributes::Password;
        let c = a + b;

        assert!(matches!(c, AccountAttributes::Specific(_)));

        let AccountAttributes::Specific(attrs) = c else { unreachable!() };
        assert_eq!(
            attrs.as_slice(),
            &[AccountAttr::Password, AccountAttr::PermissionsDocument]
        );
    }
}
