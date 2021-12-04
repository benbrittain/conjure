#![feature(array_zip)]

use {
    crate::{octree::Octree, shape::ShapeFunc},
    anyhow::{anyhow, Error},
    argh::FromArgs,
    log::info,
    std::path::PathBuf,
};

mod camera;
mod event_loop;
mod model;
mod octree;
mod render_state;
mod shape;
mod texture;

mod octant_model;

#[derive(FromArgs)]
/// Conjure shapes.
struct Arguments {
    /// input file
    #[argh(positional)]
    input: Option<PathBuf>,
}

fn main() -> Result<(), Error> {
    env_logger::init();

    info!("starting up");

    let args: Arguments = argh::from_env();
    if args.input.is_some() {
        return Err(anyhow!("no support for PL yet"));
    }

    let mut tree = Octree::new(-16.0, 16.0);

    tree.render_shape(
        1.0,
        ShapeFunc::new(move |x, y, z| {
            (((0.0 - z) * (0.0 - z)) + ((0.0 - x) * (0.0 - x)) + ((0.0 - y) * (0.0 - y))).sqrt()
                - 5.0
        }),
    );

    event_loop::start(&mut tree)
}
