#![feature(array_zip)]

use {
    crate::{octree::Octree, shape::CsgFunc},
    argh::FromArgs,
    log::info,
    std::path::PathBuf,
    winit::{event_loop::EventLoop, platform::unix::WindowBuilderExtUnix, window::WindowBuilder},
};

mod camera;
mod dual_contour;
mod event_loop;
mod lang;
mod model;
mod octree;
mod render_state;
mod shape;
mod texture;
mod types;
mod util;

#[derive(FromArgs)]
/// Conjure shapes.
struct Arguments {
    /// input file
    #[argh(positional)]
    input: PathBuf,

    /// resolution of the rendered model
    #[argh(option)]
    resolution: f32,

    /// size of the space the model is rendered into
    /// (-bound .. bound)
    #[argh(option)]
    bound: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("starting up");

    // Read in command line args
    let args: Arguments = argh::from_env();

    // Evaluate the contents of the file
    let contents = std::fs::read_to_string(args.input)?;
    let tokens = lang::Reader::read_str(&contents)?;
    let env = lang::Env::new();
    let core_ns = lang::Namespace::new();
    for (sym, func) in core_ns.into_iter() {
        env.register_sym(sym, func);
    }
    let ast = lang::eval(tokens, &env)?;

    // Contour the output CSG function
    let mut tree = Octree::new(-args.bound, args.bound);

    if let crate::lang::Ty::CsgFunc(csg_func) = ast {
        tree.render_shape(args.resolution, csg_func);
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("conjure")
        .with_app_id("conjure".to_string())
        .build(&event_loop)?;

    // Render the shape
    event_loop::start(window, event_loop, &mut tree)
}
