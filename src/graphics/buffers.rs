use std::rc::Rc;

use crate::graphics::opengl::GLIndexBuffer;
use crate::graphics::opengl::GLVertexBuffer;

#[derive(Debug, Clone, PartialEq)]
pub struct VertexBuffer {
    pub(crate) handle: Rc<GLVertexBuffer>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexBuffer {
    pub(crate) handle: Rc<GLIndexBuffer>,
}
