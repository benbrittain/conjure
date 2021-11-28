use {
    anyhow::{anyhow, Error},
    argh::FromArgs,
    std::path::PathBuf,
};

#[derive(FromArgs)]
/// Conjure shapes.
struct Arguments {
    /// input file
    #[argh(positional)]
    input: Option<PathBuf>,
}

fn main() -> Result<(), Error> {
    let args: Arguments = argh::from_env();
    if args.input.is_some() {
        return Err(anyhow!("no support for PL yet"));
    }
    Ok(())
}
