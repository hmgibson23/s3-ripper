use super::Region;

pub type StorageTypes = Vec<String>;

#[derive(Debug)]
pub struct Bucket {
    pub name: String,

    pub region: Option<Region>,

    pub storage_types: Option<StorageTypes>,
}

pub type Buckets = Vec<Bucket>;
