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

    let mut tree = Octree::new(-8.0, 8.0);

    tree.render_shape(
        0.1,
        ShapeFunc::new(move |x, y, z| {
            f32::max(
                z - 10.0,
                f32::max(
                    10.0 - z,
                    f32::max(10.0 - y, f32::max(y - 10.0, f32::max(10.0 - x, x - 10.0))),
                ),
            )
        }),
    );
    event_loop::start()
}
