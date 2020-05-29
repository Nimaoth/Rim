
#[macro_export]
macro_rules! GL {
    ($expression:expr) => {
        unsafe {
            use gl::*;
            $expression;
            loop {
                let gl_error = gl::GetError();
                if gl_error == gl::NO_ERROR {
                    break;
                }

                println!("[OpenGL] Error: {}", gl_error);
            }
        }
    };
}
