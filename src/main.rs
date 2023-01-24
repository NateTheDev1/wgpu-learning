mod camera;
mod instance;
mod model;
mod resources;
mod state;
mod texture;
mod vertex;
mod window_adapter;

fn main() {
    // Main is not async: window_adapter::run();
    pollster::block_on(window_adapter::run());
}
