use std::path::PathBuf;

use magicpak::action;
use magicpak::base::Result;
use magicpak::domain::{Bundle, Executable};

use log::error;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "magicpak")]
struct Opt {
    #[structopt(value_name = "INPUT", parse(from_os_str))]
    /// Input executable
    input: PathBuf,

    #[structopt(value_name = "OUTPUT", parse(from_os_str))]
    /// Output destination
    output: PathBuf,

    #[structopt(short, long, value_name = "GLOB")]
    /// Additionally include files/directories with glob patterns
    include: Vec<String>,

    #[structopt(short, long, value_name = "GLOB")]
    /// Exclude files/directories from the resulting bundle with glob patterns
    exclude: Vec<String>,

    #[structopt(long, value_name = "PATH")]
    /// Make directories in the resulting bundle
    mkdir: Vec<String>,

    #[structopt(short = "r", long, value_name = "PATH")]
    /// Specify the installation path of the executable in the bundle
    install_to: Option<String>,

    #[structopt(long, value_name = "LEVEL", default_value = "Warn", possible_values = &["Off", "Error", "Warn", "Info", "Debug"])]
    /// Specify the log level
    log_level: log::LevelFilter,

    #[structopt(short, long)]
    /// Verbose mode, same as --log-level Info
    verbose: bool,

    #[structopt(short, long)]
    /// Enable testing
    test: bool,

    #[structopt(long, value_name = "COMMAND")]
    /// Specify the test command to use in --test
    test_command: Option<String>,

    #[structopt(long, value_name = "CONTENT")]
    /// Specify stdin content supplied to the test command in --test
    test_stdin: Option<String>,

    #[structopt(long, value_name = "CONTENT")]
    /// Test stdout of the test command
    test_stdout: Option<String>,

    #[structopt(short, long)]
    /// Enable dynamic analysis
    dynamic: bool,

    #[structopt(
        long,
        value_name = "ARG",
        allow_hyphen_values = true,
        number_of_values = 1
    )]
    /// Specify arguments passed to the executable in --dynamic
    dynamic_arg: Vec<String>,

    #[structopt(long, value_name = "CONTENT")]
    /// Specify stdin content supplied to the executable in --dynamic
    dynamic_stdin: Option<String>,

    #[structopt(short, long)]
    /// Compression the executable with npx
    compress: bool,

    #[structopt(
        long,
        value_name = "ARG",
        allow_hyphen_values = true,
        number_of_values = 1
    )]
    /// Specify arguments passed to upx in --compress
    upx_arg: Vec<String>,

    #[structopt(long, value_name = "PATH or NAME", default_value = "busybox")]
    /// Specify the path or name of busybox that would be used in testing
    busybox: String,

    #[structopt(long, value_name = "PATH or NAME", default_value = "upx")]
    /// Specify the path or name of upx that would be used in compression
    upx: String,

    #[structopt(long, value_name = "PATH or NAME", default_value = "cc", env = "CC")]
    /// Specify the path or name of c compiler that would be used in
    /// the name resolution of shared library dependencies
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

    if opt.test {
        action::test(
            &bundle,
            &exe,
            opt.test_command.as_ref(),
            opt.test_stdin.as_ref(),
            opt.test_stdout.as_ref(),
            &opt.busybox,
        )?;
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
