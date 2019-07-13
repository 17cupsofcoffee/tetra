use std::rc::Rc;

use crate::graphics::opengl::{GLIndexBuffer, GLVertexBuffer};

pub struct VertexBuffer {
    pub(crate) handle: Rc<GLVertexBuffer>,
}

pub struct IndexBuffer {
    pub(crate) handle: Rc<GLIndexBuffer>,
}
