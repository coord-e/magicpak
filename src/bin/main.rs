use std::path::PathBuf;

use magicpak::action;
use magicpak::base::{Error, Result};
use magicpak::domain::{Bundle, Executable};

use clap::Parser;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[value(rename_all = "PascalCase")]
enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
}

impl LogLevel {
    fn to_level_filter(self) -> tracing_subscriber::filter::LevelFilter {
        use tracing_subscriber::filter::LevelFilter;
        match self {
            LogLevel::Off => LevelFilter::OFF,
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Debug => LevelFilter::DEBUG,
        }
    }
}

#[derive(Parser)]
#[command(name = "magicpak")]
struct Args {
    #[arg(value_name = "INPUT", required = true)]
    /// Input executable
    input: Vec<PathBuf>,

    #[arg(value_name = "OUTPUT")]
    /// Output destination
    output: PathBuf,

    #[arg(short, long, value_name = "GLOB")]
    /// Additionally include files/directories with glob patterns
    include: Vec<String>,

    #[arg(short, long, value_name = "GLOB")]
    /// Exclude files/directories from the resulting bundle with glob patterns
    exclude: Vec<String>,

    #[arg(long, value_name = "PATH")]
    /// Make directories in the resulting bundle
    mkdir: Vec<String>,

    #[arg(short = 'r', long, value_name = "PATH")]
    /// Specify the installation path of the executable in the bundle
    install_to: Option<String>,

    #[arg(long, value_name = "LEVEL", default_value = "Warn")]
    /// Specify the log level
    log_level: LogLevel,

    #[arg(short, long)]
    /// Verbose mode, same as --log-level Info
    verbose: bool,

    #[arg(short, long)]
    /// Enable testing
    test: bool,

    #[arg(long, value_name = "COMMAND")]
    /// Specify the test command to use in --test
    test_command: Option<String>,

    #[arg(long, value_name = "CONTENT")]
    /// Specify stdin content supplied to the test command in --test
    test_stdin: Option<String>,

    #[arg(long, value_name = "CONTENT")]
    /// Test stdout of the test command
    test_stdout: Option<String>,

    #[arg(short, long)]
    /// Enable dynamic analysis
    dynamic: bool,

    #[arg(
        long,
        value_name = "ARG",
        allow_hyphen_values = true,
        number_of_values = 1
    )]
    /// Specify arguments passed to the executable in --dynamic
    dynamic_arg: Vec<String>,

    #[arg(long, value_name = "CONTENT")]
    /// Specify stdin content supplied to the executable in --dynamic
    dynamic_stdin: Option<String>,

    #[arg(short, long)]
    /// Compress the executable with npx
    compress: bool,

    #[arg(
        long,
        value_name = "ARG",
        allow_hyphen_values = true,
        number_of_values = 1
    )]
    /// Specify arguments passed to upx in --compress
    upx_arg: Vec<String>,

    #[arg(long, value_name = "PATH or NAME", default_value = "busybox")]
    /// Specify the path or name of busybox that would be used in testing
    busybox: String,

    #[arg(long, value_name = "PATH or NAME", default_value = "upx")]
    /// Specify the path or name of upx that would be used in compression
    upx: String,

    #[arg(long, value_name = "PATH or NAME", default_value = "cc", env = "CC")]
    /// Specify the path or name of c compiler that would be used in
    /// the name resolution of shared library dependencies
    cc: String,
}

fn run(args: &Args) -> Result<()> {
    let mut bundle = Bundle::new();
    let mut exes = args
        .input
        .iter()
        .map(Executable::load)
        .collect::<Result<Vec<_>>>()?;

    for exe in &exes {
        action::bundle_shared_object_dependencies(&mut bundle, exe, &args.cc)?;
    }

    if args.dynamic {
        let &[ref exe] = &exes[..] else {
            return Err(Error::DynamicWithMultipleInputsUnsupported);
        };

        action::bundle_dynamic_dependencies(
            &mut bundle,
            exe,
            &args.dynamic_arg,
            args.dynamic_stdin.as_ref(),
        )?;
    }

    if args.compress {
        for exe in &mut exes {
            action::compress_exexcutable(exe, &args.upx, &args.upx_arg)?;
        }
    }

    for (exe, input) in exes.iter().zip(&args.input) {
        action::bundle_executable(&mut bundle, exe, input, args.install_to.as_ref())?;
    }

    for dir in &args.mkdir {
        action::make_directory(&mut bundle, dir);
    }

    for glob in &args.include {
        action::include_glob(&mut bundle, glob, &args.cc)?;
    }

    for glob in &args.exclude {
        action::exclude_glob(&mut bundle, glob)?;
    }

    if args.test {
        let &[ref exe] = &exes[..] else {
            return Err(Error::TestWithMultipleInputsUnsupported);
        };

        action::test(
            &bundle,
            exe,
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

    let level_filter = if args.verbose {
        tracing_subscriber::filter::LevelFilter::INFO
    } else {
        args.log_level.to_level_filter()
    };

    use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};
    tracing_subscriber::registry()
        .with(level_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_target(false)
                .without_time(),
        )
        .init();

    std::process::exit(match run(&args) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("error: {}", e);
            1
        }
    });
}
