use {
    anyhow::{anyhow, Error},
    argh::FromArgs,
    log::info,
    std::path::PathBuf,
};

mod camera;
mod event_loop;
mod model;
mod render_state;
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

    event_loop::start()
}
