use crate::config::state::AppState;

pub struct CustomerService<'a> {
    _app_state: &'a AppState,
}

impl<'a> CustomerService<'a> {
    pub fn from(_app_state: &'a AppState) -> Self {
        Self { _app_state }
    }
}
