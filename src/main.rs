use winit::event_loop::EventLoop;

mod app;
mod camera;
mod instance;
mod lighting;
mod model;
mod resources;
mod state;
mod texture;
mod vertex;
mod render_pipeline;

use app::*;

fn main() {
    run().unwrap();
}

pub fn run() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::with_user_event().build()?;

    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
