use graphics::opengl::GLProgram;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use App;

#[derive(Clone)]
pub struct Shader {
    pub(crate) handle: Rc<GLProgram>,
}

impl Shader {
    pub fn new<P: AsRef<Path>>(app: &mut App, vertex_path: P, fragment_path: P) -> Shader {
        let program = app.gl.compile_program(
            &fs::read_to_string(vertex_path).unwrap(),
            &fs::read_to_string(fragment_path).unwrap(),
        );

        Shader {
            handle: Rc::new(program),
        }
    }
}
