pub struct UpdateCredentialsInput {
    pub account_id: String,
    pub curr_password: String,
    pub new_password: String,
}

pub enum UpdateCredentialsError {
    AccountNotFound,
    WrongPassword,
    InvalidParameter,
    InternalError,
}
