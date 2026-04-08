#[derive(Default, Debug, PartialEq, Eq)]
pub enum UpdateRequest {
    #[default]
    None,
    Incremental, // Search narrowed -> filter down
    Full,        // Search widened/changed -> reset & filter
}

#[derive(Default)]
pub struct UpdateRequestTracker {
    pending_update: UpdateRequest,
}

impl UpdateRequestTracker {
    pub fn request(&mut self, update: UpdateRequest) {
        // a full update always takes priority over an incremental one
        if update == UpdateRequest::Full || self.pending_update == UpdateRequest::None {
            self.pending_update = update;
        }
    }

    /// returns the current pending update and resets it to None
    pub fn consume(&mut self) -> UpdateRequest {
        std::mem::replace(&mut self.pending_update, UpdateRequest::None)
    }
}
