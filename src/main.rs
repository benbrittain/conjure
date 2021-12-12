#![feature(array_zip)]

use {
    crate::{octree::Octree, shape::ShapeFunc},
    anyhow::{anyhow, Error},
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
    input: Option<PathBuf>,
}

fn main() -> Result<(), Error> {
    env_logger::init();

    info!("starting up");

    //for (sym, func) in core_ns.into_iter() {
    //    env.register_sym(sym, func);
    //}

    let args: Arguments = argh::from_env();
    if let Some(fin) = args.input {
        let contents = std::fs::read_to_string(fin)?;
        let tokens = lang::Reader::read_str(&contents)?;
        println!("{}", tokens);
        let env = lang::Env::new();
        let core_ns = lang::Namespace::new();
        for (sym, func) in core_ns.into_iter() {
            env.register_sym(sym, func);
        }

        dbg!(&tokens);
        let ast = lang::eval(tokens, &env)?;
        eprintln!("\n{}", ast);
        return Err(anyhow!("no support for PL yet"));
    }

    let mut tree = Octree::new(-16.0, 16.0);

    tree.render_shape(
        0.5,
        ShapeFunc::new(move |x, y, z| {
            (((0.0 - z) * (0.0 - z)) + ((0.0 - x) * (0.0 - x)) + ((0.0 - y) * (0.0 - y))).sqrt()
                - 5.0
        }),
    );

    event_loop::start(&mut tree)
}
