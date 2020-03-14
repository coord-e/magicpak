use std::path::PathBuf;

use magicpak::action;
use magicpak::domain::{Bundle, Executable};

use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "magicpak")]
struct Opt {
    #[structopt(parse(from_os_str))]
    /// Input executable
    input: PathBuf,

    #[structopt(parse(from_os_str))]
    /// Output destination
    output: PathBuf,

    #[structopt(short, long)]
    /// additionally include file/directory with glob patterns.
    include: Vec<String>,

    #[structopt(short, long)]
    /// exclude file/directory from the bundle with glob patterns.
    exclude: Vec<String>,

    #[structopt(long)]
    /// make directories in the resulting bundle
    mkdir: Vec<String>,

    #[structopt(short = "r", long)]
    /// specify installation path of the executable in the bundle
    install_to: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    let mut bundle = Bundle::new();
    let exe = Executable::load(opt.input)?;

    action::bundle_shared_object_dependencies(&mut bundle, &exe)?;
    action::bundle_executable(&mut bundle, &exe, opt.install_to)?;

    for dir in opt.mkdir {
        action::make_directory(&mut bundle, &dir);
    }

    for glob in opt.include {
        action::include_glob(&mut bundle, &glob)?;
    }

    for glob in opt.exclude {
        action::exclude_glob(&mut bundle, &glob)?;
    }

    action::emit(&mut bundle, opt.output)?;

    Ok(())
}
