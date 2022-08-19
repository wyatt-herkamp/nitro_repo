use crate::api::User;

use crate::configs::{get_user_config};
use crate::Parser;



use style_term::DefaultColor::{Green, Red};
use style_term::{StyleString};


#[derive(Debug, Parser)]
pub struct Instances {
    #[clap(default_value = "false")]
    pub skip_login: bool,
}

impl Instances {
    pub async fn execute(self) -> anyhow::Result<()> {
        let result = get_user_config()?;
        let reqwest = reqwest::ClientBuilder::new()
            .user_agent("Nitro Repository CLI")
            .build()
            .unwrap();
        for (name, instance) in result.repositories {
            println!("{}: {}", name, instance.url);

            let option = User::me(&reqwest, instance.url.clone(), &instance).await?;
            if let Some(v) = option {
                println!("{}", v.username.to_string().style().text_color(Green));
            } else {
                //TODO remove the instance
                println!("{}", "No user found.".style().text_color(Red));
            }
        }
        Ok(())
    }
}
