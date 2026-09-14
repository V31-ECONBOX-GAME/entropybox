use crate::domain::model::demo::Demo;

#[derive(Default)]
pub struct DemoRows;

impl DemoRows {
    pub fn select(&self, id: u32) -> Option<Demo> {
        Some(Demo {
            id,
            name: format!("demo-{id}"),
        })
    }

    pub fn select_page(&self, offset: u32, limit: u32) -> Vec<Demo> {
        (offset..offset + limit)
            .filter_map(|id| self.select(id))
            .collect()
    }
}
