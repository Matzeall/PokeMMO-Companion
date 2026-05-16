use crate::data_source::SwarmData;

pub mod github_gist;

pub trait DataTarget {
    fn publish_data(&self, data: &SwarmData);
}
