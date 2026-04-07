use std::mem;

/// a helper data structure to hold all kinds of data related to making a string search, without
/// knowing it's source data or making any of the work itself
pub struct SearchIndex {
    pub request_full_update: bool, // search_index needs to be widened -> reset & filter
    pub request_incremental_update: bool, // requests incremental filtering on current matches

    cur_search_prompt: String,
    pub matches: Vec<usize>,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self {
            request_full_update: false,
            request_incremental_update: false,
            cur_search_prompt: "".into(),
            matches: Vec::new(),
        }
    }
    pub fn get_search_prompt(&self) -> String {
        self.cur_search_prompt.clone()
    }

    pub fn set_search_prompt(&mut self, search_prompt: String) {
        if search_prompt == self.cur_search_prompt {
            return;
        }

        // update cur_search_prompt and save old prompt without cloning
        let old_prompt = mem::replace(&mut self.cur_search_prompt, search_prompt);

        // request incremental update only
        if self.cur_search_prompt.starts_with(&old_prompt) {
            self.request_incremental_update = true;
            return;
        }

        self.request_full_update = true;
    }
}
