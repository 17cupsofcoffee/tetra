use graphics::opengl::GLProgram;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use App;

pub const DEFAULT_VERTEX_SHADER: &str = include_str!("../resources/shader.vert");
pub const DEFAULT_FRAGMENT_SHADER: &str = include_str!("../resources/shader.frag");

#[derive(Clone)]
pub struct Shader {
    pub(crate) handle: Rc<GLProgram>,
}

impl Shader {
    pub fn new(app: &mut App, vertex_shader: &str, fragment_shader: &str) -> Shader {
        let program = app.gl.compile_program(vertex_shader, fragment_shader);

        Shader {
            handle: Rc::new(program),
        }
    }

    pub fn from_file<P: AsRef<Path>>(app: &mut App, vertex_path: P, fragment_path: P) -> Shader {
        Shader::new(
            app,
            &fs::read_to_string(vertex_path).unwrap(),
            &fs::read_to_string(fragment_path).unwrap(),
        )
    }

    pub fn default(app: &mut App) -> Shader {
        // TODO: Could we make this a lazy static?
        Shader::new(app, DEFAULT_VERTEX_SHADER, DEFAULT_FRAGMENT_SHADER)
    }
}
