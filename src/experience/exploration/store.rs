// /src/experience/exploration/store.rs

pub trait ExplorationRepository {
    fn create(&self, exploration: &Exploration) -> Result<()>;

    fn get(&self, id: &str) -> Result<Option<Exploration>>;

    fn update(&self, exploration: &Exploration) -> Result<()>;

    fn list_active(&self) -> Result<Vec<Exploration>>;
}
