use std::rc::Rc;

use crate::platform::opengl::GLIndexBuffer;
use crate::platform::opengl::GLVertexBuffer;

#[derive(Debug, Clone, PartialEq)]
pub struct VertexBuffer {
    pub(crate) handle: Rc<GLVertexBuffer>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexBuffer {
    pub(crate) handle: Rc<GLIndexBuffer>,
}
