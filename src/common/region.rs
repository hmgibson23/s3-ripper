use aws_types::region;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Region {
    region: Option<region::Region>,
}

