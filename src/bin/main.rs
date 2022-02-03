use std::path::PathBuf;

use magicpak::action;
use magicpak::base::Result;
use magicpak::domain::{Bundle, Executable};

use clap::Parser;
use log::error;

#[derive(Parser)]
#[clap(name = "magicpak")]
struct Args {
    #[clap(value_name = "INPUT", parse(from_os_str))]
    /// Input executable
    input: PathBuf,

    #[clap(value_name = "OUTPUT", parse(from_os_str))]
    /// Output destination
    output: PathBuf,

    #[clap(short, long, value_name = "GLOB")]
    /// Additionally include files/directories with glob patterns
    include: Vec<String>,

    #[clap(short, long, value_name = "GLOB")]
    /// Exclude files/directories from the resulting bundle with glob patterns
    exclude: Vec<String>,

    #[clap(long, value_name = "PATH")]
    /// Make directories in the resulting bundle
    mkdir: Vec<String>,

    #[clap(short = 'r', long, value_name = "PATH")]
    /// Specify the installation path of the executable in the bundle
    install_to: Option<String>,

    #[clap(long, value_name = "LEVEL", default_value = "Warn", possible_values = &["Off", "Error", "Warn", "Info", "Debug"])]
    /// Specify the log level
    log_level: log::LevelFilter,

    #[clap(short, long)]
    /// Verbose mode, same as --log-level Info
    verbose: bool,

    #[clap(short, long)]
    /// Enable testing
    test: bool,

    #[clap(long, value_name = "COMMAND")]
    /// Specify the test command to use in --test
    test_command: Option<String>,

    #[clap(long, value_name = "CONTENT")]
    /// Specify stdin content supplied to the test command in --test
    test_stdin: Option<String>,

    #[clap(long, value_name = "CONTENT")]
    /// Test stdout of the test command
    test_stdout: Option<String>,

    #[clap(short, long)]
    /// Enable dynamic analysis
    dynamic: bool,

    #[clap(
        long,
        value_name = "ARG",
        allow_hyphen_values = true,
        number_of_values = 1
    )]
    /// Specify arguments passed to the executable in --dynamic
    dynamic_arg: Vec<String>,

    #[clap(long, value_name = "CONTENT")]
    /// Specify stdin content supplied to the executable in --dynamic
    dynamic_stdin: Option<String>,

    #[clap(short, long)]
    /// Compress the executable with npx
    compress: bool,

    #[clap(
        long,
        value_name = "ARG",
        allow_hyphen_values = true,
        number_of_values = 1
    )]
    /// Specify arguments passed to upx in --compress
    upx_arg: Vec<String>,

    #[clap(long, value_name = "PATH or NAME", default_value = "busybox")]
    /// Specify the path or name of busybox that would be used in testing
    busybox: String,

    #[clap(long, value_name = "PATH or NAME", default_value = "upx")]
    /// Specify the path or name of upx that would be used in compression
    upx: String,

    #[clap(long, value_name = "PATH or NAME", default_value = "cc", env = "CC")]
    /// Specify the path or name of c compiler that would be used in
    /// the name resolution of shared library dependencies
    cc: String,
}

fn run(args: &Args) -> Result<()> {
    let mut bundle = Bundle::new();
    let mut exe = Executable::load(&args.input)?;

    action::bundle_shared_object_dependencies(&mut bundle, &exe, &args.cc)?;

    if args.dynamic {
        action::bundle_dynamic_dependencies(
            &mut bundle,
            &exe,
            &args.dynamic_arg,
            args.dynamic_stdin.as_ref(),
        )?;
    }

    if args.compress {
        action::compress_exexcutable(&mut exe, &args.upx, &args.upx_arg)?;
    }

    action::bundle_executable(&mut bundle, &exe, &args.input, args.install_to.as_ref())?;

    for dir in &args.mkdir {
        action::make_directory(&mut bundle, dir);
    }

    for glob in &args.include {
        action::include_glob(&mut bundle, glob)?;
    }

    for glob in &args.exclude {
        action::exclude_glob(&mut bundle, glob)?;
    }

    if args.test {
        action::test(
            &bundle,
            &exe,
            args.test_command.as_ref(),
            args.test_stdin.as_ref(),
            args.test_stdout.as_ref(),
            &args.busybox,
        )?;
    }

    action::emit(&mut bundle, &args.output)?;

    Ok(())
}

fn main() {
    let args = Args::parse();

    let log_level = if args.verbose {
        log::LevelFilter::Info
    } else {
        args.log_level
    };

    fern::Dispatch::new()
        .format(|out, message, record| out.finish(format_args!("[{}] {}", record.level(), message)))
        .level(log_level)
        .chain(std::io::stderr())
        .apply()
        .unwrap();

    std::process::exit(match run(&args) {
        Ok(()) => 0,
        Err(e) => {
            error!("error: {}", e);
            1
        }
    });
}
