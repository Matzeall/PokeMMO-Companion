pub mod discord_bot;

#[derive(Default, Clone, Debug)]
pub struct SwarmData {
    pub pokemon: String,
    pub location: String,
    pub region: String,
    pub details: String,
}

impl PartialEq for SwarmData {
    fn eq(&self, other: &Self) -> bool {
        self.pokemon == other.pokemon
            && self.location == other.location
            && self.region == other.region
            && self.details == other.details
    }
}

pub trait DataSource {
    // spawn some kind of async worker/bot to retreive the data for this source (this is optional)
    fn spawn(&mut self, async_handle: tokio::runtime::Handle) -> tokio::task::JoinHandle<()>;

    // lazily update current data and return it
    fn get_current_data(&mut self) -> SwarmData;

    fn is_reliable(&self) -> bool;
}
