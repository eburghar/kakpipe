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
use structopt::StructOpt;

fn main() -> Result<()> {
	let args = Args::from_args();
	match args.mode {
		Mode::Fifo(args) => {
			// create random fifo and socket
			let tmp_dir = env::temp_dir().join("kakpipe");
			fs::create_dir_all(&tmp_dir)?;
			let fifo_path = tmp_dir.join(mktemp(10, ".fifo"));
			let socket_path = tmp_dir.join(mktemp(10, ".sock"));
			// Create the unix fifo
			unistd::mkfifo(&fifo_path, stat::Mode::S_IRWXU)?;

			// set buffer name
			let buffer_name = if let Some(name) = &args.name {
				format!("{}", name)
			} else {
				let stamp = SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)?
					.as_secs()
					.to_string();
				if args.args.len() > 1 {
					format!("{}-{}-{}", &args.cmd, &args.args[0], &stamp)
				} else {
					format!("{}-{}", &args.cmd, &stamp)
				}
			};

			// write kakoune initialization commands to stdout
			println!(
				"edit! -fifo {fifo_path}{scroll}{readonly} *{buffer_name}*\n\
				add-highlighter -override buffer/kakpipe ranges kakpipe_color_ranges\n\
				hook buffer BufClose \\*{buffer_name}\\* %{{ nop %sh{{ rm -f {fifo_path} }} }}\n\
				try %{{ remove-hooks buffer kakpipe }}\n\
				hook -group kakpipe buffer BufReadFifo .* %{{ evaluate-commands %sh{{ kakpipe range-specs {socket_path} $kak_hook_param }} }}",
				fifo_path=fifo_path.to_str().unwrap(),
				socket_path=socket_path.to_str().unwrap(),
				buffer_name=&buffer_name,
				// readonly=if args.rw { "" } else { " -readonly"}, // BUG? apparently every buffer turn readonly after this
				readonly = "",
				scroll=if args.scroll { " -scroll" } else { "" },
			);
			// set buffer options
			for (name, value) in &args.opts {
				println!("set-option buffer {} {}", name, value);
			}

			// let stdout = fs::File::create("/tmp/daemon.out").unwrap();
			// let stderr = fs::File::create("/tmp/daemon.err").unwrap();
			let daemon = Daemonize::new()
				// .stdout(stdout)
				// .stderr(stderr)
				.working_directory(env::current_dir().unwrap());
			// Detach
			daemon.start()?;
			// Concurrently run command, output stdout and stderr to fifo and serve ranges
			block_on(fifo(args, fifo_path, socket_path))?
		}
		Mode::RangeSpecs(args) => block_on(range_specs(args))?,
		Mode::Faces(_) => block_on(faces())?,
	};
	Ok(())
}
