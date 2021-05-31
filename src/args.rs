use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Utility to display text with ansi color codes inside kakoune fifo buffers or info boxes
pub struct Args {
	#[argh(subcommand)]
	pub mode: Mode,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum Mode {
	Fifo(FifoArgs),
	RangeSpecs(RangeSpecsArgs),
	Faces(FacesArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "fifo")]
/// Return kakoune commands for opening a fifo buffer and initializing highlighters for ansi-codes, then detach itself, forward
/// command output to the fifo, and serve range-specs definitions through a unix socket that can be consumed to stdout
/// with the `range-specs` subcommand.
pub struct FifoArgs {
	/// the kakoune session
	#[argh(option, short = 's')]
	pub session: String,

	/// the fifo buffer name
	#[argh(option, short = 'n')]
	pub name: Option<String>,

	/// command stderr goes to *debug* buffer instead of fifo
	#[argh(switch, short = 'd')]
	pub debug: bool,

	/// by default buffer are readonly. turns the buffer editable
	#[argh(switch, short = 'w')]
	pub rw: bool,

	/// scroll down fifo buffer as new content arrives
	#[argh(switch, short = 'S')]
	pub scroll: bool,

   	/// command to spawn
	#[argh(positional)]
	pub cmd: String,

	// arguments of command
	#[argh(positional)]
	pub args: Vec<String>
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "range-specs")]
/// Consume all available range-specs up to a given selection range from a given unix socket.
pub struct RangeSpecsArgs {
	/// socket path to get range-specs from
	#[argh(positional)]
	pub socket: String,

	/// get range-specs up to range or all available range-specs by default
	#[argh(positional, default = "\"0.0,0.0\".to_owned()")]
	pub range: String
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "faces")]
/// Forward stdin to stdout with ansi color codes converted to kakoune face definitions
pub struct FacesArgs {}
