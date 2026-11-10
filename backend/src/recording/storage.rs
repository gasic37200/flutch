//! 녹화 파일 저장소(S3 호환, path-style). 업로드는 LiveKit Egress가 하고, 우리는 삭제만 한다.
//! SigV4를 직접 서명한다(aws-sdk가 aws-lc를 끌고 오므로. hmac·sha2는 이미 있다).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use hmac::{Hmac, KeyInit, Mac};
use sha2::{Digest, Sha256};

use crate::config::S3Config;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("storage request: {0}")]
    Http(#[from] reqwest::Error),
    #[error("storage status {0}")]
    Status(u16),
    #[error("storage not configured")]
    NotConfigured,
}

#[async_trait]
pub trait ObjectStore: Send + Sync {
    /// 객체 삭제. 이미 없으면 성공.
    async fn delete(&self, key: &str) -> Result<(), StorageError>;
}

/// 저장소 설정이 없을 때(녹화 꺼짐). 지울 객체도 없다.
pub struct NoStorage;

#[async_trait]
impl ObjectStore for NoStorage {
    async fn delete(&self, _key: &str) -> Result<(), StorageError> {
        Err(StorageError::NotConfigured)
    }
}

pub struct S3Http {
    config: S3Config,
    http: reqwest::Client,
}

impl S3Http {
    pub fn new(config: S3Config, http: reqwest::Client) -> Self {
        Self { config, http }
    }
}

#[async_trait]
impl ObjectStore for S3Http {
    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        let c = &self.config;
        let url = reqwest::Url::parse(&format!("{}/{}/{key}", c.endpoint, c.bucket))
            .map_err(|_| StorageError::NotConfigured)?;
        let host = match (url.host_str(), url.port()) {
            (Some(h), Some(p)) => format!("{h}:{p}"),
            (Some(h), None) => h.to_owned(),
            _ => return Err(StorageError::NotConfigured),
        };
        let auth = sign_delete(c, &host, url.path(), Utc::now());
        let res = self
            .http
            .delete(url)
            .header("x-amz-date", auth.amz_date)
            .header("x-amz-content-sha256", EMPTY_SHA256)
            .header("authorization", auth.authorization)
            .send()
            .await?;
        match res.status().as_u16() {
            s if (200..300).contains(&s) || s == 404 => Ok(()),
            s => Err(StorageError::Status(s)),
        }
    }
}

const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

struct Signed {
    amz_date: String,
    authorization: String,
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

fn hmac(key: &[u8], data: &str) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("hmac accepts any key length");
    mac.update(data.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn signing_key(secret: &str, date: &str, region: &str, service: &str) -> Vec<u8> {
    let k = hmac(format!("AWS4{secret}").as_bytes(), date);
    let k = hmac(&k, region);
    let k = hmac(&k, service);
    hmac(&k, "aws4_request")
}

/// 빈 본문 DELETE의 SigV4 서명. `path`는 이미 URL 인코딩된 경로(키는 `recordings/<id>.mp4` 꼴이라 인코딩할 문자가 없다).
fn sign_delete(c: &S3Config, host: &str, path: &str, now: DateTime<Utc>) -> Signed {
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date = &amz_date[..8];
    let signed_headers = "host;x-amz-content-sha256;x-amz-date";
    let canonical = format!(
        "DELETE\n{path}\n\nhost:{host}\nx-amz-content-sha256:{EMPTY_SHA256}\nx-amz-date:{amz_date}\n\n{signed_headers}\n{EMPTY_SHA256}"
    );
    let scope = format!("{date}/{}/s3/aws4_request", c.region);
    let to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{}",
        hex(Sha256::digest(canonical.as_bytes()))
    );
    let sig = hex(hmac(
        &signing_key(&c.secret_key, date, &c.region, "s3"),
        &to_sign,
    ));
    Signed {
        authorization: format!(
            "AWS4-HMAC-SHA256 Credential={}/{scope}, SignedHeaders={signed_headers}, Signature={sig}",
            c.access_key
        ),
        amz_date,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AWS SigV4 문서의 서명 키 예제.
    #[test]
    fn signing_key_matches_aws_example() {
        let k = signing_key(
            "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
            "20150830",
            "us-east-1",
            "iam",
        );
        assert_eq!(
            hex(k),
            "c4afb1cc5771d871763a393e44b703571b55cc28424d1a5e86da6ed3c154a4b9"
        );
    }
}
