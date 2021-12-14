mod args;
mod cmd;
mod mktemp;
mod range_specs;

use anyhow::{bail, Result};
use async_std::task::block_on;
use daemonize::Daemonize;
use nix::{sys::stat, unistd};
use std::{env, fs, time::SystemTime};

use crate::{
	args::{Args, Mode},
	cmd::{faces::faces, fifo::fifo, range_specs::range_specs},
	mktemp::{temp_dir, temp_file, temp_id},
};

fn main() -> Result<()> {
	let args: Args = args::from_env();
	match args.mode {
		Mode::Fifo(args) => {
			// check -D arguments are well formed
			for o in args.opts.iter() {
				if let (_, None) = args::parse_key_val(o) {
					bail!("invalid KEY=value: no `=` found in `{}`", o);
				};
			}

			// create random fifo, socket and pid files
			let base = temp_id(10);
			let tmp_dir = temp_dir("kakpipe")?;
			let fifo_path = temp_file(&tmp_dir, &base, "fifo")?;
			let socket_path = temp_file(&tmp_dir, &base, "sock")?;
			let pipe_pid_path = temp_file(&tmp_dir, &base, "pid1")?;
			let daemon_pid_path = temp_file(&tmp_dir, &base, "pid2")?;

			// Create the unix fifo
			unistd::mkfifo(&fifo_path, stat::Mode::S_IRWXU)?;

			// set buffer name
			let buffer_name = if let Some(name) = &args.name {
				name.to_owned()
			} else {
				// create a timestamp
				let stamp = SystemTime::now()
					.duration_since(SystemTime::UNIX_EPOCH)?
					.as_secs()
					.to_string();
				// use the given prefix is any
				let mut res = if let Some(prefix) = &args.prefix {
					prefix.to_owned()
				// strip path from cmd
				} else if let Some(pos) = args.cmd.rfind('/') {
					if pos + 1 < args.cmd.len() {
						args.cmd[pos + 1..].to_owned()
					} else {
						args.cmd.clone()
					}
				} else {
					args.cmd.clone()
				};
				// join all argument that are not switches...
				for arg in args.args.iter().filter(|s| !s.starts_with('-')) {
					res.push('-');
					res.push_str(arg);
				}
				// ...with the stamp
				res.push('-');
				res.push_str(&stamp);
				res
			};

			// collect arguments after 'kakpipe fifo'
			let cmd_args = env::args().skip(2).fold(String::new(), |mut a, ref b| {
				a.push(' ');
				a.push_str(b);
				a
			});

			// write kakoune initialization commands to stdout
			println!(
				"{close_buffer}\
    			hook -once global BufOpenFifo \\*{buffer_name}\\* %{{ set-option buffer kakpipe_args %{{{cmd_args}}}\n alias buffer !! kakpipe-restart }}\n\
    			edit! -fifo {fifo_path}{scroll}{readonly} *{buffer_name}*\n\
				add-highlighter -override buffer/kakpipe ranges kakpipe_color_ranges\n\
				hook -once buffer BufClose \\*{buffer_name}\\* %{{ nop %sh{{\n
        			test -f {pipe_pid_path} && pid=$(cat {pipe_pid_path}) && rm -f {pipe_pid_path} && test -n $pid && kill $pid >/dev/null 2>&1\n
            		test -f {daemon_pid_path} && pid=$(cat {daemon_pid_path}) && rm -f {daemon_pid_path} && test -n $pid && kill $pid >/dev/null 2>&1\n
					test -p {fifo_path} && rm -f {fifo_path}\n
    				test -S {socket_path} && rm -f {socket_path}\n
				}} }}\n\
				try %{{ remove-hooks buffer kakpipe }}\n\
				hook -group kakpipe buffer BufReadFifo .* %{{ evaluate-commands %sh{{ test -S {socket_path} && kakpipe range-specs {socket_path} $kak_hook_param }} }}",
				close_buffer= if args.close {"delete-buffer\n"} else { ""},
				fifo_path=fifo_path.to_str().unwrap(),
				socket_path=socket_path.to_str().unwrap(),
				pipe_pid_path=pipe_pid_path.to_str().unwrap(),
				daemon_pid_path=daemon_pid_path.to_str().unwrap(),
				buffer_name=&buffer_name,
				cmd_args=&cmd_args,
				readonly=if args.rw { "" } else { " -readonly"},
				scroll=if args.scroll { " -scroll" } else { "" },
			);
			// set buffer options
			args.opts
				.iter()
				.filter_map(|s| match args::parse_key_val(s) {
					(name, Some(value)) => Some((name, value)),
					_ => None,
				})
				.for_each(|(name, value)| println!("set-option buffer {} {}", name, value));

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
			let res = block_on(fifo(args, &fifo_path, &pipe_pid_path, &socket_path));

			// at this point fifo is closed. remove it
			let _ = fs::remove_file(fifo_path);
			// remove silently temp files
			let _ = fs::remove_file(socket_path);
			let _ = fs::remove_file(pipe_pid_path);
			let _ = fs::remove_file(daemon_pid_path);

			res?
		}
		Mode::RangeSpecs(args) => block_on(range_specs(args))?,
		Mode::Faces(_) => block_on(faces())?,
	};
	Ok(())
}
