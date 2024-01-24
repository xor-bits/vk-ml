use std::{fmt, sync::OnceLock};

//

pub mod vk;

//

pub struct Handle;

//

pub enum Backend {
    Vk(vk::Backend),
    // Cpu,
}

impl fmt::Debug for Backend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Backend::Vk(_) => f.write_str("Vk"),
        }
    }
}

//

pub fn backend() -> &'static Backend {
    static BACKEND: OnceLock<Backend> = OnceLock::new();

    BACKEND.get_or_init(|| Backend::Vk(vk::new_backend().unwrap()))
}
