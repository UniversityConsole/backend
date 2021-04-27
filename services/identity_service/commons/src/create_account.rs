pub struct CreateAccountInput {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub gov_id: String,
    pub password: String,
}

pub struct CreateAccountOutput {
    pub account_id: String,
}
pub enum CreateAccountError {
    DuplicateAccount,
}
