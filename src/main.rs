use glfw::{Context, fail_on_errors};
mod renderer_backend;
use renderer_backend::state::State;

const WIDTH: u32 = 1000;
const HEIGHT: u32 = 1000;
async fn run_async() {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    let (mut window, events) = glfw
        .create_window(WIDTH, HEIGHT, "WGPU Project", glfw::WindowMode::Windowed)
        .unwrap();

    window.set_key_polling(true);
    window.make_current();

    // create state
    let mut state = State::new(&mut window).await;

    while !state.window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Size(width, height) => {
                    state.resize(width as u32, height as u32);
                }
                _ => {}
            }
        }

        state.render();
    }
}

fn main() {
    pollster::block_on(run_async());
}
