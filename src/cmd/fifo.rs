use anyhow::{anyhow, Context, Result};
use async_process::{Command, Stdio};
use async_std::{
	fs::{self, File, OpenOptions},
	io::{
		prelude::{BufReadExt, ReadExt, WriteExt},
		BufReader,
	},
	os::unix::net::UnixListener,
	prelude::FutureExt,
	stream::StreamExt,
	sync::Mutex,
};
use kak::{
	command::Client,
	range::{Pos, Range, Selection},
};
use std::{convert::TryFrom, env, path::PathBuf, sync::Arc};
use yew_ansi::get_sgr_segments;

use crate::{
	args::{self, FifoArgs},
	range_specs::SharedRanges,
};

/// Serve all accumulated range_specs definition to stdout through a unix socket
pub async fn range_specs(socket: PathBuf, sync: Arc<Mutex<SharedRanges>>) -> Result<()> {
	let listener = UnixListener::bind(&socket)
		.await
		.context("error listening to range_specs socket")?;
	let mut incoming = listener.incoming();
	let mut response = String::new();

	while let Some(stream) = incoming.next().await {
		let mut stream = stream?;

		// wait for the range to be sent by client
		let mut len = [0u8; 1];
		stream.read_exact(&mut len).await?;
		let mut range = vec![0u8; len[0] as usize];
		stream.read_exact(&mut range).await?;

		// this is the selection kakoune wants the ranges for
		let selection = String::from_utf8_lossy(&range).parse::<Selection>()?;
		// eprintln!("selection: {}", &selection);

		// reuse the same string between connexion
		response.clear();
		let fifo_end;
		let mut empty_ranges;
		// release the lock after this block
		{
			let mut sync = sync.lock().await;
			// get fifo state and ranges emptyness
			fifo_end = sync.fifo_end;
			empty_ranges = sync.ranges.is_empty();
			if !empty_ranges {
				// return all accumulated ranges lower or equal to selection
				let mut i = 0;
				for range in sync.ranges.iter() {
					// don't serve ranges for lines that kakoune didn't display yet
					if selection.is_valid() && range.selection.1 > selection.1 {
						break;
					}
					// separate ranges by space
					if i != 0 {
						response.push(' ');
					}
					response.push_str(&format!("{}", range));
					i += 1;
				}
				// remove ranges already sent (for_each consumes the iterator and the drained items)
				sync.ranges.drain(0..i).for_each(|_| ());
				// reevaluate ranges emptyness to stop the loop if fifo has been closed
				empty_ranges = sync.ranges.is_empty();
			}
		}
		// eprintln!("response: {}", &response);
		stream.write_all(response.as_bytes()).await?;
		if fifo_end && empty_ranges {
			break;
		}
	}

	// remove the socket file now that all ranges have been consumed and fifo has been closed
	fs::remove_file(socket).await?;
	Ok(())
}

/// Forward stdin to stdout after removing all ansi color codes. Range_specs are written in a data structure
/// shared between tasks
pub async fn stdin_fifo(
	args: &FifoArgs,
	fifo: PathBuf,
	pid: PathBuf,
	client: &mut Client,
	sync: Arc<Mutex<SharedRanges>>,
) -> Result<()> {
	// set environment
	let envs = args
		.vars
		.iter()
		.filter_map(|s| match args::parse_key_val(s) {
			(name, Some(value)) => Some((name, value.to_owned())),
			(name, None) => env::var(name).ok().map(|value| (name, value)),
		});

	let mut fifo_file = OpenOptions::new().write(true).open(fifo).await?;

	// async read from command and async write to fifo
	let mut cmd = Command::new(&args.cmd);
	if args.clear_env {
    	cmd.env_clear();
	}
	let child = cmd
		.envs(envs)
		.args(&args.args)
		.stderr(Stdio::piped())
		.stdout(Stdio::piped())
		.current_dir(env::current_dir().unwrap())
		.spawn();

	// write error message to fifo in case spawn failed
	if let Err(e) = child {
		fifo_file.write_all(format!("error running {}: {}", &args.cmd, e).as_bytes()).await?;
		fifo_file.flush().await?;
		return Err(e.into())
	}

	// unwrap is safe at it this point because of the return above
	let mut child = child.unwrap();

	// write the pid of the spawn process to a file then dispose it
	{
		let mut pid_file = File::create(pid).await?;
		pid_file
			.write_all(child.id().to_string().as_bytes())
			.await?;
	}

	let mut stdout_reader = BufReader::new(child.stdout.take().unwrap()).lines();
	let mut stderr_reader = BufReader::new(child.stderr.take().unwrap()).lines();
	let mut l = 1; // line number
	let mut start = 1; // column
	if args.debug {
		// stdout goes to fifo
		let fifo_task = async {
			// TODO: how to deal with ansi-codes that spans several lines ?
			while let Some(line) = stdout_reader.next().await {
				let line = line?;
				for (effect, txt) in get_sgr_segments(&line) {
					let len = u32::try_from(txt.len()).unwrap();
					let end = if len > 1 {
						start + len - 1
					} else {
						start + len
					};
					if let Some(range) = Range::new(Selection(Pos(l, start), Pos(l, end)), effect) {
						let mut sync = sync.lock().await;
						sync.ranges.push_back(range);
					}
					let _ = fifo_file.write_all(txt.as_bytes()).await;
					start = if len > 1 { end + 1 } else { end };
				}
				let _ = fifo_file.write_all(b"\n").await;
				let _ = fifo_file.flush().await;
				l += 1;
				start = 1;
			}
			Ok::<(), anyhow::Error>(())
		};

		// stderr goes to debug buffer
		let debug_task = async {
			while let Some(line) = stderr_reader.next().await {
				let line = line?;
				// debug buffer doesn't support markup otherwise we would have inserted faces
				// to get colored debug outputs
				client
					.send_command(&format!("echo -debug {}", line))
					.await?;
			}
			Ok::<(), anyhow::Error>(())
		};

		// read from stdout and stderr concurrently
		fifo_task.try_race(debug_task).await?;
	} else {
		// TODO: deduplicate code with generics ?
		// TODO: we probably lose one line if both stdout and stderr completes at the same time
		// reads from stdout and stderr simultaneously
		while let Some(line) = stdout_reader.next().race(stderr_reader.next()).await {
			let line = line?;
			for (effect, txt) in get_sgr_segments(&line) {
				let len = u32::try_from(txt.len()).unwrap();
				let end = if len > 1 {
					start + len - 1
				} else {
					start + len
				};
				if let Some(range) = Range::new(Selection(Pos(l, start), Pos(l, end)), effect) {
					let mut sync = sync.lock().await;
					sync.ranges.push_back(range);
				}
				let _ = fifo_file.write_all(txt.as_bytes()).await;
				start = if len > 1 { end + 1 } else { end };
			}
			let _ = fifo_file.write_all(b"\n").await;
			let _ = fifo_file.flush().await;
			l += 1;
			start = 1;
			// println!("{}", &line);
		}
	}

	// signal the range_specs task that we have processed all output
	{
		let _ = fifo_file.sync_all().await;
		let mut sync = sync.lock().await;
		sync.fifo_end = true;
	}

	if l >= 1 {
		Ok(())
	} else {
		// return an error to stop all tasks as there is nothing more to do
		Err(anyhow!("no output"))
	}
}

/// Print kakoune initialization command for displaying the corresponding fifo buffer then
/// combine stdin_fifo and range_specs
pub async fn fifo(args: FifoArgs, fifo: PathBuf, pid: PathBuf, socket: PathBuf) -> Result<()> {
	// client connection to kakoune session
	let mut client = Client::new(&args.session)?;
	if args.debug {
		client
			.send_command(&format!(
				"echo -debug +++ start {} {:?}",
				&args.cmd, &args.args
			))
			.await?;
	}
	// unless we use a double queue with an actor model and an atomic operation for switching
	// queues after adding or removing ranges, mutex seems to be unavoidable, because we can
	// produce and consume ranges at the same time and both task needs write access.
	let sync = Arc::new(Mutex::new(SharedRanges::new()));
	let task_stdin_fifo = stdin_fifo(&args, fifo, pid, &mut client, Arc::clone(&sync));
	let task_ranges_specs = range_specs(socket, Arc::clone(&sync));
	// stops as soon as one future fails
	task_stdin_fifo.try_join(task_ranges_specs).await?;
	if args.debug {
		client
			.send_command(&format!(
				"echo -debug +++ end {} {:?}",
				&args.cmd, &args.args
			))
			.await?;
	}
	Ok(())
}
