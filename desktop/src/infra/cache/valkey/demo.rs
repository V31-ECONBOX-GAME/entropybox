use crate::domain::model::demo::Demo;
use std::collections::HashMap;

#[derive(Default)]
pub struct DemoEntries {
    entries: HashMap<u32, Demo>,
}

impl DemoEntries {
    pub fn get(&self, id: u32) -> Option<&Demo> {
        self.entries.get(&id)
    }

    pub fn put(&mut self, demo: Demo) {
        self.entries.insert(demo.id, demo);
    }
}
