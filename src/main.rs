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
	let args: Args = args::from_env();
	match args.mode {
		Mode::Fifo(args) => {
			// check -D arguments are well formed
			for o in args.opts.iter() {
				args::parse_key_val(o)?;
			}

			// create random fifo and socket
			let tmp_dir = env::temp_dir().join("kakpipe");
			fs::create_dir_all(&tmp_dir)?;
			let tmp_id = mktemp(10);
			let mut fifo_path = tmp_dir.join(&tmp_id);
			fifo_path.set_extension("fifo");
			let mut socket_path = tmp_dir.join(&tmp_id);
			socket_path.set_extension("sock");
			let mut pipe_pid_path = tmp_dir.join(&tmp_id);
			pipe_pid_path.set_extension("pid1");
			let mut daemon_pid_path = tmp_dir.join(&tmp_id);
			daemon_pid_path.set_extension("pid2");

			// Create the unix fifo
			unistd::mkfifo(&fifo_path, stat::Mode::S_IRWXU)?;

			// set buffer name
			let buffer_name = if let Some(name) = &args.name {
				name.to_owned()
			} else {
				let stamp = SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)?
					.as_secs()
					.to_string();
				// use given prefix is any
				let mut res = if let Some(prefix) = &args.prefix {
    				prefix.to_owned()
    			// strip cmd name of dirname
    			} else if let Some(pos) = args.cmd.rfind('/') {
    				if pos + 1 < args.cmd.len() {
        				args.cmd[pos+1..].to_owned()
    				} else {
        				args.cmd.clone()
    				}
				} else {
					args.cmd.clone()
				};
				// join all argument that are not switch
				for arg in args.args.iter().filter(|s| !s.starts_with('-')) {
					res.push('-');
					res.push_str(arg);
				}
				res.push('-');
				res.push_str(&stamp);
				res
			};

			// write kakoune initialization commands to stdout
			println!(
				"edit! -fifo {fifo_path}{scroll}{readonly} *{buffer_name}*\n\
				add-highlighter -override buffer/kakpipe ranges kakpipe_color_ranges\n\
				hook buffer BufClose \\*{buffer_name}\\* %{{ nop %sh{{\n
					test -p {fifo_path} && rm -f {fifo_path}\n
    				test -S {socket_path} && rm -f {socket_path}\n
        			test -f {pipe_pid_path} && pid=$(cat {pipe_pid_path}) && rm -f {pipe_pid_path} && test -n $pid && kill $pid\n
            		test -f {daemon_pid_path} && pid=$(cat {daemon_pid_path}) && rm -f {daemon_pid_path} && test -n $pid && kill $pid\n
				}} }}\n\
				try %{{ remove-hooks buffer kakpipe }}\n\
				hook -group kakpipe buffer BufReadFifo .* %{{ evaluate-commands %sh{{ kakpipe range-specs {socket_path} $kak_hook_param }} }}",
				fifo_path=fifo_path.to_str().unwrap(),
				socket_path=socket_path.to_str().unwrap(),
				pipe_pid_path=pipe_pid_path.to_str().unwrap(),
				daemon_pid_path=daemon_pid_path.to_str().unwrap(),
				buffer_name=&buffer_name,
				// readonly=if args.rw { "" } else { " -readonly"}, // BUG? apparently every buffer turn readonly after this
				readonly = "",
				scroll=if args.scroll { " -scroll" } else { "" },
			);
			// set buffer options
			args.opts.iter().for_each(|o| {
				// unwrap is ok because we checked errors upfront
				let (name, value) = args::parse_key_val(o).unwrap();
				println!("set-option buffer {} {}", name, value);
			});

			// let stdout = fs::File::create("/tmp/daemon.out").unwrap();
			// let stderr = fs::File::create("/tmp/daemon.err").unwrap();
			let daemon = Daemonize::new()
				// .stdout(stdout)
				// .stderr(stderr)
				.pid_file(&daemon_pid_path)
				.working_directory(env::current_dir().unwrap());
			// Detach
			daemon.start()?;
			// Concurrently run command, output stdout and stderr to fifo and serve ranges
			block_on(fifo(args, fifo_path, pipe_pid_path, socket_path))?
		}
		Mode::RangeSpecs(args) => block_on(range_specs(args))?,
		Mode::Faces(_) => block_on(faces())?,
	};
	Ok(())
}
