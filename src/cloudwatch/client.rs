use crate::common::Bucket;
use anyhow::Result;
use aws_sdk_cloudwatch::client::Client as CloudWatchClient;
use aws_sdk_cloudwatch::model::{Dimension, DimensionFilter, Metric, StandardUnit, Statistic};
use aws_sdk_cloudwatch::output::GetMetricStatisticsOutput;
use aws_sdk_cloudwatch::types::DateTime;
use aws_smithy_types_convert::date_time::DateTimeExt;
use chrono::prelude::DateTime as ChronoDt;
use chrono::prelude::Utc;
use chrono::Duration;

pub struct Client {
    pub client: CloudWatchClient,
    pub bucket_name: Option<String>,
}

impl Client {
    pub async fn new(region: Option<aws_sdk_cloudwatch::Region>) -> Self {
        let config = aws_config::from_env().region(region.clone()).load().await;

        let client = CloudWatchClient::new(&config);

        Self {
            client,
            bucket_name: None,
        }
    }

    pub async fn get_metric_statistics_objects(
        &self,
        bucket: &Bucket
        ) -> Result<Vec<GetMetricStatisticsOutput>> {

        // These are used repeatedly while looping, just prepare them once.
        let now: ChronoDt<Utc> = Utc::now();
        let one_day = Duration::days(1);
        let period = one_day.num_seconds() as i32;
        let start_time = DateTime::from_chrono_utc(now - (one_day * 2));

        let storage_types = match &bucket.storage_types {
            Some(st) => st.to_owned(),
            None => Vec::new(),
        };

        let mut outputs = Vec::new();

            let dimensions = vec![
                Dimension::builder()
                    .name("BucketName")
                    .value(bucket.name.to_owned())
                    .build(),
                Dimension::builder()
                    .name("StorageType")
                    .value("AllStorageTypes")
                    .build(),
            ];

            let input = self
                .client
                .get_metric_statistics()
                .end_time(DateTime::from_chrono_utc(now))
                .metric_name("NumberOfObjects")
                .namespace("AWS/S3")
                .period(period)
                .set_dimensions(Some(dimensions))
                .start_time(start_time)
                .statistics(Statistic::Average);


            let output = input.send().await?;

            outputs.push(output);

        Ok(outputs)
    }

    pub async fn get_metric_statistics(
        &self,
        bucket: &Bucket,
        metric_name: &str
    ) -> Result<Vec<GetMetricStatisticsOutput>> {

        // These are used repeatedly while looping, just prepare them once.
        let now: ChronoDt<Utc> = Utc::now();
        let one_day = Duration::days(1);
        let period = one_day.num_seconds() as i32;
        let start_time = DateTime::from_chrono_utc(now - (one_day * 2));

        let storage_types = match &bucket.storage_types {
            Some(st) => st.to_owned(),
            None => Vec::new(),
        };

        let mut outputs = Vec::new();

        for storage_type in storage_types {
            let dimensions = vec![
                Dimension::builder()
                    .name("BucketName")
                    .value(bucket.name.to_owned())
                    .build(),
                Dimension::builder()
                    .name("StorageType")
                    .value(storage_type.to_owned())
                    .build(),
            ];

            let input = self
                .client
                .get_metric_statistics()
                .end_time(DateTime::from_chrono_utc(now))
                .metric_name(metric_name)
                .namespace("AWS/S3")
                .period(period)
                .set_dimensions(Some(dimensions))
                .start_time(start_time)
                .statistics(Statistic::Average)
                .unit(StandardUnit::Bytes);


            let output = input.send().await?;

            outputs.push(output);
        }

        Ok(outputs)
    }

    pub async fn list_metrics(&self, metric_name: &str) -> Result<Vec<Metric>> {
        let mut metrics = Vec::new();
        let mut next_token = None;

        // If we selected a bucket to list, filter for it here.
        let dimensions = match self.bucket_name.as_ref() {
            Some(bucket_name) => {
                let filter = DimensionFilter::builder()
                    .name("BucketName")
                    .value(bucket_name.to_owned())
                    .build();

                Some(vec![filter])
            }
            None => None,
        };

        loop {
            let output = self
                .client
                .list_metrics()
                .namespace("AWS/S3")
                .metric_name(metric_name)
                .set_dimensions(dimensions.clone())
                .set_next_token(next_token)
                .send()
                .await?;

            if let Some(m) = output.metrics {
                metrics.append(&mut m.clone());
            }

            match output.next_token {
                Some(t) => next_token = Some(t),
                None => break,
            };
        }


        Ok(metrics)
    }
}
