use super::{Bucket, Buckets};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait BucketSizer {
    async fn buckets(&self) -> Result<Buckets>;

    async fn bucket_size(&self, bucket: &Bucket) -> Result<u64>;

    async fn bucket_objects(&self, bucket: &Bucket) -> Result<u64>;
}
