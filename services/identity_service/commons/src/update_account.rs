pub struct UpdateAccountInput {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub gov_id: String,
    pub password: String,
}

pub struct UpdateAccountOutput {
    pub account_id: String,
}

pub enum UpdateAccountError {
    AccountNotFound,
    InvalidParameter,
    InternalError,
}
