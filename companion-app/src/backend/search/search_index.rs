use super::{data_bridge::DataBridge, search_update::UpdateRequest};
use std::{
    mem,
    sync::Arc,
    time::{Duration, Instant},
};

const TRANSLATION_UPDATE_INTERVAL: Duration = Duration::from_millis(500);

/// a helper data structure to hold results & manage searching some Vec<AnyBaseData>, without
/// knowing it's source data or making any of the work itself
pub struct SearchIndex<B: DataBridge> {
    pub data_bridge: B,
    pub matches: Vec<usize>,
    last_updated_search: Instant,
}

impl<B: DataBridge> SearchIndex<B> {
    pub fn new(data_bridge: B) -> Self {
        Self {
            data_bridge,
            matches: Vec::new(),
            last_updated_search: Instant::now(),
        }
    }

    pub fn update_search_index(&mut self) {
        // update user search index in intervals
        if Instant::now().duration_since(self.last_updated_search) > TRANSLATION_UPDATE_INTERVAL {
            // handle a data_bridges update request
            match self.data_bridge.update_request().consume() {
                UpdateRequest::None => (),
                UpdateRequest::Incremental => self.filter_search_index_down(),
                UpdateRequest::Full => self.update_search_index_full(),
            };
        }
    }

    fn update_search_index_full(&mut self) {
        let full_matches = self.data_bridge.get_all_potential_matches();
        self.matches = self.data_bridge.filter_matches(full_matches);
    }

    fn filter_search_index_down(&mut self) {
        self.matches = self
            .data_bridge
            .filter_matches(mem::take(&mut self.matches));
    }

    pub fn get_search_results(&self) -> Vec<Arc<B::Item>> {
        self.data_bridge.get_items(&self.matches)
    }
}
