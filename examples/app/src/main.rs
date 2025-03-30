use xs_backend::Backend;
mod app;

pub fn main() {
    let mut backend = Backend::new(1280, 720);
    backend.log("test");
    backend.run(|| {
            app::sidebar()
    });
}
