use crate::common::{BucketNames, StorageTypes};
use aws_sdk_cloudwatch::model::Metric;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
pub struct BucketMetrics(pub HashMap<String, StorageTypes>);

impl BucketMetrics {
    pub fn bucket_names(&self) -> BucketNames {
        self.0.iter().map(|(k, _v)| k.to_string()).collect()
    }

    pub fn storage_types(&self, bucket: &str) -> &StorageTypes {
        self.0.get(bucket).unwrap()
    }
}

impl From<Vec<Metric>> for BucketMetrics {
    fn from(metrics: Vec<Metric>) -> Self {
        let mut bucket_metrics = HashMap::new();

        for metric in metrics {
            let dimensions = match metric.dimensions {
                Some(d) => d,
                None => continue,
            };

            let mut name = String::new();
            let mut storage_type = String::new();

            for dimension in dimensions {
                let dname = match dimension.name {
                    Some(n) => n,
                    None => continue,
                };

                match dname.as_ref() {
                    "BucketName" => name = dimension.value.unwrap(),
                    "StorageType" => storage_type = dimension.value.unwrap(),
                    _ => {}
                }
            }

            let storage_types = bucket_metrics.entry(name).or_insert_with(StorageTypes::new);

            storage_types.push(storage_type);
        }

        BucketMetrics(bucket_metrics)
    }
}
