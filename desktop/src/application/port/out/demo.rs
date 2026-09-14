use crate::domain::model::demo::Demo;

pub trait DemoPort {
    fn find(&self, id: u32) -> Option<Demo>;

    fn list(&self, offset: u32, limit: u32) -> Vec<Demo>;
}
