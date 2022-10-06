use anyhow::Result;
mod common;
use common::BucketSizer;
mod cloudwatch;
use humansize::{format_size, DECIMAL};
use std::env;
use aws_types::region;

struct Client(Box<dyn BucketSizer>);

impl Client {
    async fn new(region: Option<aws_sdk_cloudwatch::Region>) -> Self {
        let client: Box<dyn BucketSizer> = {
            let client = cloudwatch::Client::new(region);
            Box::new(client.await)
        };

        Client(client)
    }

    async fn do_fetch(&self) -> Result<()> {
        let buckets = self.0.buckets().await?;

        let mut total_size: u64 = 0;

        for bucket in buckets {
            let size = self.0.bucket_size(&bucket).await?;
            let number_of_objects = self.0.bucket_objects(&bucket).await?;

            total_size += size;

            println!(
                "{size}\t{bucket}\t{number_of_objects}",
                size = format_size(size, DECIMAL),
                bucket = bucket.name,
                number_of_objects = number_of_objects
            );
        }

        println!("{size}\t.", size = format_size(total_size, DECIMAL));

        Ok(())
    }
}

fn set_region() -> Option<aws_sdk_cloudwatch::Region> {
    let possibilities = vec![env::var("AWS_REGION"), env::var("AWS_DEFAULT_REGION")];

    let region = possibilities
        .iter()
        .find_map(|region| region.as_ref().ok())
        .map(|region| region::Region::new(region.to_owned()));

    region
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Lists all buckets.");

    let region = set_region();
    let client = Client::new(region).await;
    match client.do_fetch().await {
        Ok(_) => println!("Done."),
        Err(x) => println!("{:?}", x)
    }

    Ok(())
}
