use std::rc::Rc;

use crate::platform::IndexBufferHandle;
use crate::platform::VertexBufferHandle;

#[derive(Debug, Clone, PartialEq)]
pub struct VertexBuffer {
    pub(crate) handle: Rc<VertexBufferHandle>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexBuffer {
    pub(crate) handle: Rc<IndexBufferHandle>,
}
