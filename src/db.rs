use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};
use once_cell::sync::Lazy;
use reqwest::{
    header::{HeaderMap, HeaderName},
    Client, IntoUrl, Url,
};
use serde_json::{Map, Value};

use crate::err::{Error, Result};

static OK: Lazy<Value> = Lazy::new(|| Value::String("OK".into()));

pub type QueryResponse = Vec<Vec<Map<String, Value>>>;

#[derive(Debug, Clone)]
pub struct Database {
    client: Client,
    headers: HeaderMap,
    url: Url,
}
impl Database {
    pub async fn new(
        client: Client,
        url: impl IntoUrl,
        namespace: &str,
        database: &str,
        credentials: &str,
    ) -> Result<Database> {
        let encoder = BASE64_STANDARD_NO_PAD;
        let credentials = encoder.encode(credentials);
        let auth_str = format!("Basic {}", credentials);
        let headers: HeaderMap = [
            ("Accept", "application/json"),
            ("NS", namespace),
            ("DB", database),
            ("Authorization", &auth_str),
        ]
        .into_iter()
        .map(|(k, v)| {
            (
                HeaderName::from_bytes(k.as_bytes()).unwrap(),
                v.parse().unwrap(),
            )
        })
        .collect();
        let url = url.into_url()?;
        let request = client.post(url.clone()).headers(headers.clone());
        let res = request.body("INFO FOR DB;").send().await?;
        if !res.status().is_success() {
            return Err(Error::BadHttpStatus(res.status()));
        }
        let mut body: Vec<Map<String, Value>> = res.json().await?;
        let map = body.remove(0);
        if map.get("status").as_ref() == Some(&&*OK) {
            return Ok(Database {
                client,
                headers,
                url,
            });
        }
        Err(Error::DatabaseLogin(Value::Object(map)))
    }
    /// Makes a SurrealQL query to the database.
    /// If any statement fails, an [Error] is returned, likely BadQuery.
    /// If every statement succeeds, a [Vec] with the results of each statement is returned.
    /// The result of each statement is a [Vec] of objects.
    pub async fn query(&self, req: String) -> Result<QueryResponse> {
        let base = self
            .client
            .post(self.url.clone())
            .headers(self.headers.clone());
        let res = base.body(req).send().await?.json().await?;
        let checked = match res {
            Value::Object(obj) => match obj.get("information") {
                Some(msg) => Err(Error::BadQuery(msg.clone())),
                None => Err(Error::BadQuery("incomprehensible response".into())),
            },
            Value::Array(a) => Ok(a.into_iter().map(|statement| match statement {
                Value::Object(mut map) => {
                    let result = map.remove("result");
                    let status = map.get("status");
                    match (status, result) {
                        (Some(ok), Some(Value::Array(arr))) if ok == &*OK => {
                            Ok(arr.into_iter().map(|rvalue| match rvalue {
                                Value::Object(map) => Ok(map),
                                _ => Err(Error::BadQuery("database improperly configured".into())),
                            }))
                        }
                        (Some(_err), _) => Err(Error::BadQuery(map.into())),
                        (None, _) => Err(Error::BadQuery("no status given".into())),
                    }
                }
                _ => Err(Error::BadQuery("not an object".to_string().into())),
            })),
            _ => Err(Error::BadQuery("incomprehensible response".into())),
        }?;
        let mut out = Vec::new();
        for statement_response in checked {
            let response = statement_response?;
            let mut sub_out = Vec::new();
            for item in response {
                sub_out.push(item?);
            }
            out.push(sub_out)
        }
        Ok(out)
    }
}

pub fn validate_id(s: &str) -> Result<()> {
    if s.contains('`') {
        return Err(Error::BadString(s.to_string()));
    }
    Ok(())
}

pub fn validate_str(s: &str) -> Result<()> {
    if s.contains('\'') {
        return Err(Error::BadString(s.to_string()));
    }
    Ok(())
}

pub fn check_obj_status(obj: &Map<String, Value>) -> bool {
    obj.get("status") == Some(&OK)
}
