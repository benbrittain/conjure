#![feature(array_zip)]

use {
    crate::{octree::Octree, shape::CsgFunc},
    argh::FromArgs,
    log::info,
    std::path::PathBuf,
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
    let mut tree = Octree::new(-16.0, 16.0);

    if let crate::lang::Ty::CsgFunc(csg_func) = ast {
        tree.render_shape(0.5, csg_func);
    }

    // Render the shape
    event_loop::start(&mut tree)
}
