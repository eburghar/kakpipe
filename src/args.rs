use argh::{FromArgs, TopLevelCommand};
use std::path::Path;

/// Utility to display text with ansi color codes inside kakoune fifo buffers or info boxes
#[derive(FromArgs)]
pub struct Args {
	#[argh(subcommand)]
	pub mode: Mode,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum Mode {
	Fifo(FifoArgs),
	RangeSpecs(RangeSpecsArgs),
	Faces(FacesArgs),
}

/// Split a variable definition in NAME and VALUE
pub(crate) fn parse_key_val(exp: &str) -> (&str, Option<&str>) {
	let i = exp.find("=");
	if let Some(i) = i {
		// something after =
		if i + 1 < exp.len() {
			(&exp[..i], Some(&exp[i + 1..]))
		// nothing after =
		} else {
			(&exp[..i], Some(""))
		}
	} else {
		(exp, None)
	}
}

/// Return kakoune commands for opening a fifo buffer and initializing highlighters for ansi-codes, then detach itself, forward
/// command output to the fifo, and serve range-specs definitions through a unix socket that can be consumed to stdout
/// with the `range-specs` subcommand.
#[derive(FromArgs)]
#[argh(subcommand, name = "fifo")]
pub struct FifoArgs {
	/// turns the buffer editable. by default they are readonly
	#[argh(switch, short = 'w')]
	pub rw: bool,

	/// scroll down fifo buffer as new content arrives
	#[argh(switch, short = 'S')]
	pub scroll: bool,

	/// stderr goes to *debug* buffer instead of fifo
	#[argh(switch, short = 'd')]
	pub debug: bool,

	/// kakoune session
	#[argh(option, short = 's')]
	pub session: String,

	/// fifo buffer name prefix (default is the command name)
	#[argh(option, short = 'N')]
	pub prefix: Option<String>,

	/// fifo buffer name (default is prefix + args + timestamp)
	#[argh(option, short = 'n')]
	pub name: Option<String>,

	/// clear environment
	#[argh(switch, short = 'k')]
	pub clear_env: bool,

	/// environment variables to set (NAME=VALUE)
	#[argh(option, short = 'V')]
	pub vars: Vec<String>,

	/// options to set in the buffer scope (NAME=VALUE)
	#[argh(option, short = 'D')]
	pub opts: Vec<String>,

	/// command to spawn
	#[argh(positional)]
	pub cmd: String,

	// arguments of command
	#[argh(positional)]
	pub args: Vec<String>,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "range-specs")]
/// Consume all available range-specs up to a given selection range from a given unix socket.
pub struct RangeSpecsArgs {
	/// socket path to get range-specs from
	#[argh(positional)]
	pub socket: String,

	/// get range-specs up to range or all available range-specs by default
	#[argh(positional, default = "\"0.0,0.0\".to_owned()")]
	pub range: String,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "faces")]
/// Forward stdin to stdout with ansi color codes converted to kakoune face definitions
pub struct FacesArgs {}

fn cmd<'a>(default: &'a String, path: &'a String) -> &'a str {
	Path::new(path)
		.file_name()
		.map(|s| s.to_str())
		.flatten()
		.unwrap_or(default.as_str())
}

/// copy of argh::from_env to insert command name and version
pub fn from_env<T: TopLevelCommand>() -> T {
	const NAME: &'static str = env!("CARGO_PKG_NAME");
	const VERSION: &'static str = env!("CARGO_PKG_VERSION");
	let strings: Vec<String> = std::env::args().collect();
	let cmd = cmd(&strings[0], &strings[0]);
	let strs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
	T::from_args(&[cmd], &strs[1..]).unwrap_or_else(|early_exit| {
		println!("{} {}\n", NAME, VERSION);
		println!("{}", early_exit.output);
		std::process::exit(match early_exit.status {
			Ok(()) => 0,
			Err(()) => 1,
		})
	})
}
