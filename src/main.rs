mod args;
mod cmd;
mod mktemp;
mod range_specs;

use crate::{
	args::{Args, Mode},
	cmd::{faces::faces, fifo::fifo, range_specs::range_specs},
	mktemp::mktemp,
};
use anyhow::Result;
use async_std::task::block_on;
use daemonize::Daemonize;
use nix::{sys::stat, unistd};
use std::{env, fs, time::SystemTime};

fn main() -> Result<()> {
	let args: Args = argh::from_env();
	match args.mode {
		Mode::Fifo(args) => {
			let tmp_dir = env::temp_dir().join("kakpipe");
			fs::create_dir_all(&tmp_dir)?;
			let fifo_path = tmp_dir.join(mktemp(".fifo"));
			let socket_path = tmp_dir.join(mktemp(".sock"));
			let buffer_name = if let Some(name) = &args.name {
				format!("{}", name)
			} else {
				let stamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs().to_string();
				if args.args.len() > 1 {
					format!("{}-{}-{}", &args.cmd, &args.args[0], &stamp)
				} else {
					format!("{}-{}", &args.cmd, &stamp)
				}
			};

			// Create the unix fifo
			unistd::mkfifo(&fifo_path, stat::Mode::S_IRWXU)?;

			// Send kakoune initialization commands on stdout
			println!(
				"edit! -fifo {fifo_path}{scroll} *{buffer_name}*\n\
				hook buffer BufClose .* %{{ nop %sh{{ rm -f {fifo_path} }} }}\n\
				add-highlighter buffer/kakpipe ranges kakpipe_color_ranges\n\
				hook -group kakpipe buffer BufReadFifo .* %{{ evaluate-commands -draft %sh{{ kakpipe range-specs {socket_path} $kak_hook_param }} }}\n\
				hook -group kakpipe buffer BufCloseFifo .* %{{ evaluate-commands -draft %sh{{ kakpipe range-specs {socket_path} }} }}",
				fifo_path=fifo_path.to_str().unwrap(),
				socket_path=socket_path.to_str().unwrap(),
				buffer_name=&buffer_name,
				scroll=if args.scroll { " -scroll" } else { "" }
			);

			// let stdout = fs::File::create("/tmp/daemon.out").unwrap();
			// let stderr = fs::File::create("/tmp/daemon.err").unwrap();
			let daemon = Daemonize::new()
				// .stdout(stdout)
				// .stderr(stderr)
				.working_directory(env::current_dir().unwrap());
			// Detach
			daemon.start()?;
			// Concurrently output to fifo and serve ranges
			block_on(fifo(args, fifo_path, socket_path))?
		}
		Mode::RangeSpecs(args) => block_on(range_specs(args))?,
		Mode::Faces(_) => block_on(faces())?,
	};
	Ok(())
}
