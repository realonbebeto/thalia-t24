use anyhow::Context;
use aws_config::BehaviorVersion;
use aws_sdk_s3::{Client, Error as S3Error, presigning::PresigningConfig, primitives::ByteStream};
use aws_types::region::Region;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
pub struct S3Client {
    pub client: Arc<Client>,
    pub bucket: String,
}

impl S3Client {
    pub async fn default(region: String, bucket: String) -> Self {
        let region = Region::new(region);
        let config = aws_config::defaults(BehaviorVersion::v2025_08_07())
            .region(region)
            .load()
            .await;
        let client = Arc::new(Client::new(&config));

        Self { client, bucket }
    }

    pub async fn upload_to_s3(
        &self,
        file: Vec<u8>,
        bucket: Arc<String>,
        key: String,
    ) -> Result<(), S3Error> {
        let body = ByteStream::from(file);

        self.client
            .put_object()
            .bucket(bucket.as_str())
            .key(key)
            .body(body)
            .send()
            .await?;
        Ok(())
    }

    pub async fn delete_from_s3(&self, bucket: &str, key: &str) -> Result<(), anyhow::Error> {
        self.client
            .delete_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .context("Failed to delete file on s3")?;
        Ok(())
    }

    pub async fn fetch_presigned_uri(
        &self,
        bucket: &str,
        key: &str,
        duration: u64,
    ) -> Result<String, anyhow::Error> {
        let expiration = PresigningConfig::expires_in(Duration::from_secs(duration))
            .context("Failed to build s3 presigning config")?;

        let uri = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .presigned(expiration)
            .await
            .context("Failed to build a presigned link to the s3 object")?
            .uri()
            .to_string();

        Ok(uri)
    }
}
