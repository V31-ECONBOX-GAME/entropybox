use crate::application::dto::demo::DemoQuery;
use crate::domain::model::demo::Demo;

pub trait DemoUseCase {
    fn get(&self, id: u32) -> Option<Demo>;

    fn page(&self, query: &DemoQuery) -> Vec<Demo>;
}
