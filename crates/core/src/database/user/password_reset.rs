use crate::database::DateTime;

pub struct UserPasswordReset {
    pub id: u32,
    pub user_id: i32,
    pub token: String,
    pub expires_at: DateTime,
    pub used_at: DateTime,
    pub created: DateTime,
}
