#![feature(array_zip)]

use {
    crate::shape::CsgFunc,
    argh::FromArgs,
    log::info,
    notify::{watcher, RecursiveMode, Watcher},
    std::{path::PathBuf, sync::mpsc::channel, time::Duration},
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
pub struct Arguments {
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

fn eval_ast(input: PathBuf) -> Result<crate::lang::Ty, Box<dyn std::error::Error>> {
    // Slurp the contents of the file
    let contents = std::fs::read_to_string(input)?;

    // Parse
    let tokens = lang::Reader::read_str(&contents)?;
    let env = lang::Env::new();
    let core_ns = lang::Namespace::new();
    for (sym, func) in core_ns.into_iter() {
        env.register_sym(sym, func);
    }

    // Eval
    let ast = lang::eval(tokens, &env)?;
    Ok(ast)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("starting up");

    // Read in command line args
    let args: Arguments = argh::from_env();

    let event_loop = EventLoop::new();
    let proxy = event_loop.create_proxy();
    let window = WindowBuilder::new()
        .with_title("conjure")
        .with_app_id("conjure".to_string())
        .build(&event_loop)?;

    let (tx, rx) = channel();
    let (ast_sender, ast_recv) = channel();

    let mut watcher = watcher(tx, Duration::from_micros(10))?;
    watcher.watch(args.input.parent().unwrap(), RecursiveMode::Recursive)?;

    let ast = eval_ast(args.input.clone())?;
    if let crate::lang::Ty::CsgFunc(csg_func) = ast {
        ast_sender.send(csg_func)?;
        proxy.send_event(())?;
    }

    let input = args.input.clone().canonicalize()?;
    std::thread::spawn(move || loop {
        if let Ok(notify::DebouncedEvent::Create(path)) = rx.recv() {
            if let Ok(path) = path.canonicalize() {
                if path == input {
                    let ast = eval_ast(path).unwrap();
                    if let crate::lang::Ty::CsgFunc(csg_func) = ast {
                        let _ = ast_sender.send(csg_func);
                        let _ = proxy.send_event(());
                    }
                }
            }
        }
    });

    // Render the shape
    event_loop::start(window, event_loop, ast_recv, args)
}
