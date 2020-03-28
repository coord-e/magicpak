use std::path::PathBuf;

use magicpak::action;
use magicpak::base::Result;
use magicpak::domain::{Bundle, Executable};

use log::error;
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

    #[structopt(long, default_value = "Warn", possible_values = &["Off", "Error", "Warn", "Info", "Debug"])]
    /// specify log output level
    log_level: log::LevelFilter,

    #[structopt(short, long)]
    /// verbose mode. same as --log-level Info
    verbose: bool,

    #[structopt(short, long)]
    /// enable testing
    test: Option<String>,

    #[structopt(short, long)]
    /// enable dynamic analysis
    dynamic: bool,

    #[structopt(long, allow_hyphen_values = true, number_of_values = 1)]
    /// arguments passed to the executable in dynamic analysis
    dynamic_arg: Vec<String>,

    #[structopt(long)]
    /// arguments passed to the executable in dynamic analysis
    dynamic_stdin: Option<String>,

    #[structopt(short, long)]
    /// enable compression
    compress: bool,

    #[structopt(long, allow_hyphen_values = true, number_of_values = 1)]
    /// arguments passed to upx in --compress
    upx_arg: Vec<String>,

    #[structopt(long, default_value = "busybox")]
    /// specify the path or name of busybox that would be used in testing.
    busybox: String,

    #[structopt(long, default_value = "upx")]
    /// specify the path or name of upx that would be used in compression.
    upx: String,

    #[structopt(long, default_value = "cc", env = "CC")]
    /// specify the path or name of c compiler that would be used in
    /// the name resolution of shared library dependencies.
    cc: String,
}

fn run(opt: &Opt) -> Result<()> {
    let mut bundle = Bundle::new();
    let mut exe = Executable::load(&opt.input)?;

    action::bundle_shared_object_dependencies(&mut bundle, &exe, &opt.cc)?;

    if opt.dynamic {
        action::bundle_dynamic_dependencies(
            &mut bundle,
            &exe,
            &opt.dynamic_arg,
            opt.dynamic_stdin.as_ref(),
        )?;
    }

    if opt.compress {
        action::compress_exexcutable(&mut exe, &opt.upx, &opt.upx_arg)?;
    }

    action::bundle_executable(&mut bundle, &exe, &opt.input, opt.install_to.as_ref())?;

    for dir in &opt.mkdir {
        action::make_directory(&mut bundle, &dir);
    }

    for glob in &opt.include {
        action::include_glob(&mut bundle, &glob)?;
    }

    for glob in &opt.exclude {
        action::exclude_glob(&mut bundle, &glob)?;
    }

    if let Some(command) = &opt.test {
        action::test(&bundle, &command, &opt.busybox)?;
    }

    action::emit(&mut bundle, &opt.output)?;

    Ok(())
}

fn main() {
    let opt = Opt::from_args();

    let log_level = if opt.verbose {
        log::LevelFilter::Info
    } else {
        opt.log_level
    };

    fern::Dispatch::new()
        .format(|out, message, _| out.finish(format_args!("[magicpak] {}", message)))
        .level(log_level)
        .chain(std::io::stderr())
        .apply()
        .unwrap();

    std::process::exit(match run(&opt) {
        Ok(()) => 0,
        Err(e) => {
            error!("error: {}", e);
            1
        }
    });
}
