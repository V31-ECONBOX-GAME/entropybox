use crate::application::port::r#in::demo::DemoUseCase;
use crate::presentation::dto::demo::DemoResponse;

pub fn show(use_case: &impl DemoUseCase, id: u32) -> Option<DemoResponse> {
    use_case.get(id).map(DemoResponse::from)
}
