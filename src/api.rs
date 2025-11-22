use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct User {
    pub unique_identifier: String,
    pub display_name: String,
}

pub struct BrightspaceApi {
    client: Client,
    base_url: String,
    access_token: String,
}

impl BrightspaceApi {
    pub fn new(
        base_url: String,
        access_token: String,
    ) -> Self {
        Self {
            client: Client::new(),
            base_url,
            access_token,
        }
    }

    pub async fn whoami(&self) -> Result<User> {
        let url = format!("{}/d2l/api/lp/1.30/users/whoami", self.base_url);
        let user = self
            .client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await?
            .json::<User>()
            .await?;
        Ok(user)
    }
}
