use argon2::Argon2;
use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};
use rand::{thread_rng, Rng};
use serde_json::Value;
use std::str;

use crate::{
    db::{validate_str, Database},
    err::{Error, Result},
};

#[derive(Debug, Clone)]
pub struct UserManager {
    db: Database,
}
impl UserManager {
    pub fn new(db: Database) -> Self {
        UserManager { db }
    }
    // basically exists for webfinger
    pub async fn get_local_actor(&self, acct: &str) -> Result<String> {
        let username = if acct.starts_with("acct:") {
            acct[5..]
                .split('@')
                .next()
                .ok_or_else(|| Error::NoSuchEntity)?
        } else {
            return Err(Error::NoSuchEntity);
        }
        .to_string();
        validate_name(&username)?;
        let query = format!(
            "\
            SELECT id FROM user:`{username}`\
        "
        );

        let response = self
            .db
            .query(query)
            .await?
            .pop()
            .ok_or_else(|| Error::BadQuery("empty response".into()))?;
        if response.len() == 0 {
            return Err(Error::NoSuchEntity);
        }

        Ok(username)
    }
    pub async fn create_user(&self, name: &str, password: &str) -> Result<String> {
        validate_name(name)?;
        validate_str(password)?;

        // big computation. maybe move to blocking thread?
        let mut rng = thread_rng();
        let mut salt = [0u8; 256];
        rng.fill(&mut salt);

        let argon = Argon2::default();
        let mut hash = [0u8; 256];
        argon
            .hash_password_into(password.as_bytes(), &salt, &mut hash)
            .map_err(|_| Error::HashFailure)?;

        let encoder = BASE64_STANDARD_NO_PAD;
        let mut salt_base64 = [0u8; 342];
        encoder.encode_slice(&salt, &mut salt_base64).unwrap();
        let mut hash_base64 = [0u8; 342];
        encoder.encode_slice(&hash, &mut hash_base64).unwrap();

        let salt_str = str::from_utf8(&salt_base64).unwrap();
        let hash_str = str::from_utf8(&hash_base64).unwrap();
        let query = format!(
            "CREATE user:`{name}` SET
                salt = '{salt_str}',
                pwd_hash = '{hash_str}'
                RETURN id"
        );
        let response = self
            .db
            .query(query)
            .await?
            .pop()
            .and_then(|mut res| res.pop())
            .ok_or_else(|| Error::BadQuery("empty response".into()))?;
        let id = response
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::BadQuery("id missing".into()))?;
        Ok(id.to_string())
    }
    pub async fn change_password(&self, name: &str, password: &str) -> Result<()> {
        todo!()
    }
}

pub fn validate_name(s: &str) -> Result<()> {
    if s.chars().any(|c| c.is_whitespace() || c == '`') {
        return Err(Error::BadString(s.to_string()));
    }
    Ok(())
}
