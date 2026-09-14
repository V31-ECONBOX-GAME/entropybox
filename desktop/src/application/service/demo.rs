use crate::application::dto::demo::DemoQuery;
use crate::application::port::out::demo::DemoPort;
use crate::application::port::r#in::demo::DemoUseCase;
use crate::domain::model::demo::Demo;

pub struct DemoService<P> {
    port: P,
}

impl<P: DemoPort> DemoService<P> {
    pub fn new(port: P) -> Self {
        Self { port }
    }
}

impl<P: DemoPort> DemoUseCase for DemoService<P> {
    fn get(&self, id: u32) -> Option<Demo> {
        self.port.find(id)
    }

    fn page(&self, query: &DemoQuery) -> Vec<Demo> {
        self.port.list(query.page * query.size, query.size)
    }
}
