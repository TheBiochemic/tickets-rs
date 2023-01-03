
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone)]
pub enum BucketPanelLocationType {
    All,
    Reset,
    Filter,
    Adapter,
    Entry
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct BucketPanelLocation {
    pub entry_type: BucketPanelLocationType,
    pub adapter: String,
    pub section: String
}