use std::collections::HashMap;
use uuid::Uuid;
use crate::domain::{Task, TaskFilter};
pub trait TaskQuery {
    fn filter_tasks(&mut self, tasks: &HashMap<Uuid, Task>, filter: &TaskFilter) -> Vec<Uuid>;
}
pub struct CachedTaskQuery { cache: HashMap<String, Vec<Uuid>> }
impl CachedTaskQuery {
    pub fn new() -> Self { Self { cache: HashMap::new() } }
    pub fn invalidate(&mut self) { self.cache.clear(); }
}
impl TaskQuery for CachedTaskQuery {
    fn filter_tasks(&mut self, _tasks: &HashMap<Uuid, Task>, _filter: &TaskFilter) -> Vec<Uuid> { todo!() }
}
