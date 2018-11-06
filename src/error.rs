pub type Result<T = ()> = std::result::Result<T, TetraError>;

#[derive(Debug)]
pub enum TetraError {
    Io(std::io::Error),
    Sdl(String),
    OpenGl(String),
    Image(image::ImageError),
}
