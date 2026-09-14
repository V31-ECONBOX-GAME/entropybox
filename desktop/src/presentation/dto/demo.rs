use crate::domain::model::demo::Demo;

#[derive(Debug, Clone, PartialEq)]
pub struct DemoResponse {
    pub id: u32,
    pub label: String,
}

impl From<Demo> for DemoResponse {
    fn from(demo: Demo) -> Self {
        Self {
            id: demo.id,
            label: demo.name,
        }
    }
}
