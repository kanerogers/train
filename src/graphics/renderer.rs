use std::sync::Arc;

use super::context::Context;

pub struct Renderer;
impl Renderer {
    pub(crate) fn new(context: Arc<Context>) -> Self {
        Self
    }
}
