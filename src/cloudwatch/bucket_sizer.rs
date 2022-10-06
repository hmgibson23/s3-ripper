use super::bucket_metrics::BucketMetrics;
use super::client::Client;
use crate::common::{Bucket, BucketSizer, Buckets};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use aws_smithy_types_convert::date_time::DateTimeExt;

#[async_trait]
impl BucketSizer for Client {
    async fn buckets(&self) -> Result<Buckets> {

        let metrics: BucketMetrics = self.list_metrics("BucketSizeBytes").await?.into();

        let mut buckets = Buckets::new();

        for bucket in metrics.bucket_names() {
            let storage_types = metrics.storage_types(&bucket).to_owned();

            let bucket = Bucket {
                name: bucket,
                region: None,
                storage_types: Some(storage_types),
            };

            buckets.push(bucket);
        }

        Ok(buckets)
    }

    async fn bucket_objects(&self, bucket: &Bucket) -> Result<u64> {
        let mut number_of_objects = 0;
        let metric_statistics = self
            .get_metric_statistics_objects(bucket)
            .await?;
        for stats in metric_statistics {
            let mut datapoints = match stats.datapoints{
                Some(d) => d,
                None => continue,
            };
            datapoints.sort_by(|a, b| {
                let a_timestamp = a.timestamp.unwrap().to_chrono_utc();
                let b_timestamp = b.timestamp.unwrap().to_chrono_utc();

                b_timestamp.cmp(&a_timestamp)
            });
            if datapoints.is_empty() {
                return Err(anyhow!("Failed to fetch any CloudWatch datapoints!"));
            };

           let datapoint = &datapoints[0];
            let b = datapoint.average.expect("Could't unwrap average");

            number_of_objects += b.round() as u64;
        }

        Ok(number_of_objects)
    }

    async fn bucket_size(&self, bucket: &Bucket) -> Result<u64> {
        let mut size = 0;

        let metric_statistics = self
            .get_metric_statistics(bucket, "BucketSizeBytes")
            .await?;
        for stats in metric_statistics {
            let mut datapoints = match stats.datapoints {
                Some(d) => d,
                None => continue,
            };

            //cloudwatch might be empty
            if datapoints.is_empty() {
                return Err(anyhow!("Failed to fetch any CloudWatch datapoints!"));
            };

            datapoints.sort_by(|a, b| {
                let a_timestamp = a.timestamp.unwrap().to_chrono_utc();
                let b_timestamp = b.timestamp.unwrap().to_chrono_utc();

                b_timestamp.cmp(&a_timestamp)
            });

            let datapoint = &datapoints[0];

            let bytes = datapoint.average.expect("Could't unwrap average");

            size += bytes.round() as u64;
        }


        Ok(size)
    }
}
