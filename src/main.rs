use {
    argh::FromArgs,
    conjure::{event_loop, lang, shape::CsgFunc},
    log::info,
    nalgebra::DMatrix,
    notify::{watcher, RecursiveMode, Watcher},
    std::{path::PathBuf, sync::mpsc::channel, sync::Arc, time::Duration},
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

mod mars;

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

fn eval_ast(input: PathBuf) -> Result<conjure::lang::Ty, Box<dyn std::error::Error>> {
    // Slurp the contents of the file
    let contents = std::fs::read_to_string(input)?;

    // Parse
    let tokens = lang::Reader::read_str(&contents)?;
    let env = lang::Env::new();
    let mut core_ns = lang::Namespace::new();

    core_ns.add_function("mars", |_li| Ok(crate::lang::Ty::CsgFunc(mars::mars_func())));

    // Add namespace to environment
    for (sym, func) in core_ns.into_iter() {
        env.register_sym(sym, func);
    }

    // Eval
    let ast = lang::eval(tokens, &env)?;
    Ok(ast)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let latlng = mars::Polar::new(-10.0, 359.0, 30.0);
    dbg!(&latlng);
    let xyz: mars::Cartesian = latlng.into();
    dbg!(&xyz);
    let nlatlng: mars::Polar = xyz.into();
    dbg!(&nlatlng);

    return Ok(());

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

    let mut watcher = watcher(tx, Duration::from_micros(4))?;
    watcher.watch(args.input.parent().unwrap(), RecursiveMode::Recursive)?;

    let ast = eval_ast(args.input.clone())?;
    if let conjure::lang::Ty::CsgFunc(csg_func) = ast {
        let csg_func: CsgFunc =
            Arc::try_unwrap(csg_func).expect("No refrences to the ast should remain");
        ast_sender.send(csg_func)?;
        proxy.send_event(())?;
    }

    let input = args.input.clone().canonicalize()?;
    std::thread::spawn(move || loop {
        if let Ok(notify::DebouncedEvent::Create(path)) = rx.recv() {
            if let Ok(path) = path.canonicalize() {
                if path == input {
                    let ast = eval_ast(path).unwrap();
                    if let conjure::lang::Ty::CsgFunc(csg_func) = ast {
                        let csg_func: CsgFunc = Arc::try_unwrap(csg_func)
                            .expect("No refrences to the ast should remain");
                        let _ = ast_sender.send(csg_func);
                        let _ = proxy.send_event(());
                    }
                }
            }
        }
    });

    let depth = ((args.bound * 2.0) / args.resolution).log2() as u8;
    eprintln!("Rendering a shape at a resolution of {} (depth: {})", args.resolution, depth);
    // Render the shape
    event_loop::start(window, event_loop, ast_recv, args.resolution, args.bound)
}
