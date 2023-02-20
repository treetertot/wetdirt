use axum::Json;
use serde::Serialize;

use crate::{actor::UserManager, err::Result};

#[derive(Debug, Clone)]
pub struct WebFinger {
    manager: UserManager,
    domain: String,
}
impl WebFinger {
    pub async fn lookup(&self, acct: &str) -> Result<Json<User>> {
        let parsed_name = self.manager.get_local_actor(acct).await?;
        let domain = &self.domain;

        Ok(Json(User {
            subject: format!("acct:{parsed_name}@{domain}"),
            links: vec![LinkPack {
                rel: "self".to_string(),
                typ: "application/activity+json".to_string(),
                href: format!("https://{domain}/users/{parsed_name}"),
            }],
        }))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct User {
    subject: String,
    links: Vec<LinkPack>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LinkPack {
    rel: String,
    #[serde(rename = "type")]
    typ: String,
    href: String,
}
