use structopt::StructOpt;

/// Utility to display text with ansi color codes inside kakoune fifo buffers or info boxes
#[derive(StructOpt)]
#[structopt()]
pub struct Args {
	#[structopt(subcommand)]
	pub mode: Mode,
}

#[derive(StructOpt)]
pub enum Mode {
	Fifo(FifoArgs),
	RangeSpecs(RangeSpecsArgs),
	Faces(FacesArgs),
}

/// Return kakoune commands for opening a fifo buffer and initializing highlighters for ansi-codes, then detach itself, forward
/// command output to the fifo, and serve range-specs definitions through a unix socket that can be consumed to stdout
/// with the `range-specs` subcommand.
#[derive(StructOpt)]
pub struct FifoArgs {
	/// by default buffer are readonly. turns the buffer editable
	#[structopt(long, short = "w")]
	pub rw: bool,

	/// scroll down fifo buffer as new content arrives
	#[structopt(long, short = "S")]
	pub scroll: bool,

	/// stderr goes to *debug* buffer instead of fifo
	#[structopt(long, short = "d")]
	pub debug: bool,

	/// kakoune session
	#[structopt(long, short = "s")]
	pub session: String,

	/// fifo buffer name
	#[structopt(long, short = "n")]
	pub name: Option<String>,

   	/// command to spawn
	pub cmd: String,

	// arguments of command
	pub args: Vec<String>
}

#[derive(StructOpt)]
/// Consume all available range-specs up to a given selection range from a given unix socket.
pub struct RangeSpecsArgs {
	/// socket path to get range-specs from
	pub socket: String,

	/// get range-specs up to range or all available range-specs by default
	#[structopt(default_value = "0.0,0.0")]
	pub range: String
}

#[derive(StructOpt)]
/// Forward stdin to stdout with ansi color codes converted to kakoune face definitions
pub struct FacesArgs {}
