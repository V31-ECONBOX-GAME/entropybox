use crate::domain::model::demo::Demo;
use crate::infra::cache::valkey::demo::DemoEntries;

#[derive(Default)]
pub struct DemoCacheAdapter {
    entries: DemoEntries,
}

impl DemoCacheAdapter {
    pub fn cached(&mut self, demo: Demo) -> Demo {
        self.entries.put(demo.clone());
        demo
    }

    pub fn hit(&self, id: u32) -> Option<&Demo> {
        self.entries.get(id)
    }
}
