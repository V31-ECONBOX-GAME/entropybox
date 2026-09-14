use crate::application::port::out::demo::DemoPort;
use crate::domain::model::demo::Demo;
use crate::infra::persistence::sqllite::demo::DemoRows;

#[derive(Default)]
pub struct DemoPersistenceAdapter {
    rows: DemoRows,
}

impl DemoPort for DemoPersistenceAdapter {
    fn find(&self, id: u32) -> Option<Demo> {
        self.rows.select(id)
    }

    fn list(&self, offset: u32, limit: u32) -> Vec<Demo> {
        self.rows.select_page(offset, limit)
    }
}
