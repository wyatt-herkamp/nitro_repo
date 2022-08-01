use crate::configs::user::RepositoryInstance;
use crate::configs::{get_user_config, save_user_config};
use crate::Parser;
use inquire::{Text};
use serde::{Deserialize};
use serde_json::json;
use style_term::DefaultColor::{Green, Red};
use style_term::{StyleString};
use uuid::Uuid;

#[derive(Debug, Parser)]
pub struct AddInstance {
    /// The URL for the Nitro Repository Instance
    url: String,
    name: String,
}
#[derive(Deserialize, Clone, Debug)]
pub struct NewTokenResponse {
    pub token: String,
    pub token_id: Uuid,
}

impl AddInstance {
    pub async fn execute(self) -> anyhow::Result<()> {
        let username = Text::new("Username").prompt()?;
        let password = Text::new("Password").prompt()?;
        let reqwest = reqwest::ClientBuilder::new()
            .user_agent("Nitro Repository CLI")
            .build()
            .unwrap();
        let value = json! ({
            "username": username,
            "password": password,
        });
        let url = if self.url.ends_with('/') {
            self.url.trim_end_matches('/').to_string()
        } else {
            self.url
        };
        println!("{}", url);
        let response = reqwest
            .post(format!("{}/api/token/create", url))
            .json(&value)
            .send()
            .await?;
        if response.status().is_success() {
            let response = response.json::<NewTokenResponse>().await?;
            println!(
                "{}",
                format!(
                    "Token: {}; Token ID {}",
                    &response.token, &response.token_id
                )
                .style()
                .text_color(Green)
            );
            let mut user_config = get_user_config()?;
            let repository = RepositoryInstance {
                url,
                token: response.token.clone(),
                name: self.name.clone(),
                token_uuid: response.token_id,
            };
            user_config.repositories.insert(self.name, repository);
            save_user_config(&user_config)?;
        } else {
            println!(
                "{}, Error: {}",
                "Failed to add instance".style().text_color(Red),
                response.status()
            );
        }
        Ok(())
    }
}
