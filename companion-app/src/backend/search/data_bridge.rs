use super::search_update::UpdateRequestTracker;
use std::sync::Arc;

pub trait DataBridge {
    // each bridge has it's own associated element type it operates on
    type Item;

    /// used to reset the search_index matches list
    fn get_all_potential_matches(&self) -> Vec<usize>;

    fn filter_matches(&self, cur_matches: Vec<usize>) -> Vec<usize>;

    fn get_items(&self, indices: &[usize]) -> Vec<Arc<Self::Item>>;

    /// the returned UpdateRequestTracker is meant to be consumed() when the requested update to
    /// the search_index is made
    fn update_request(&mut self) -> &mut UpdateRequestTracker;
}
