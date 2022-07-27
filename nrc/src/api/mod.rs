use reqwest::header::{HeaderValue, AUTHORIZATION};
use reqwest::Client;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub username: String,
    pub email: String,
    pub permissions: UserPermissions,
    pub created: i64,
}
impl User {
    pub async fn me(
        client: &Client,
        url: impl AsRef<str>,
        auth: &impl Auth,
    ) -> anyhow::Result<Option<User>> {
        let response = client
            .get(format!("{}/api/me", url.as_ref()))
            .header(AUTHORIZATION, auth.get_as_header())
            .send()
            .await?;
        if response.status().is_success() {
            let user = response.json::<User>().await?;
            Ok(Some(user))
        } else if response.status().eq(&reqwest::StatusCode::UNAUTHORIZED) {
            Ok(None)
        } else {
            Err(anyhow::anyhow!("Failed to get user {}", response.status()))
        }
    }
}
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Default)]
pub struct UserPermissions {
    pub disabled: bool,
    pub admin: bool,
    pub user_manager: bool,
    pub repository_manager: bool,
    pub deployer: Option<RepositoryPermission>,
    pub viewer: Option<RepositoryPermission>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct RepositoryPermission {
    pub permissions: Vec<String>,
}

pub trait Auth {
    fn get_as_header(&self) -> HeaderValue;
}
