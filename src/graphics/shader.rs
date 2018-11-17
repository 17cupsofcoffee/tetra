use std::fs;
use std::path::Path;
use std::rc::Rc;

use error::{Result, TetraError};
use graphics::opengl::GLProgram;
use Context;

#[derive(Clone)]
pub struct Shader {
    pub(crate) handle: Rc<GLProgram>,
}

impl Shader {
    pub fn new(ctx: &mut Context, vertex_shader: &str, fragment_shader: &str) -> Result<Shader> {
        // TODO: If this fails, we need to actually return an error instead of crashing
        let program = ctx.gl.compile_program(vertex_shader, fragment_shader);

        Ok(Shader {
            handle: Rc::new(program),
        })
    }

    pub fn from_file<P: AsRef<Path>>(
        ctx: &mut Context,
        vertex_path: P,
        fragment_path: P,
    ) -> Result<Shader> {
        Shader::new(
            ctx,
            &fs::read_to_string(vertex_path).map_err(TetraError::Io)?,
            &fs::read_to_string(fragment_path).map_err(TetraError::Io)?,
        )
    }
}
