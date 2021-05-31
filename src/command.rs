//! Incomplete client that can send command directly to the kakoune control socket

use anyhow::{Context, Result};
use async_std::{io::prelude::WriteExt, os::unix::net::UnixStream};
use byteorder::{NativeEndian, WriteBytesExt};
use std::{convert::TryFrom, env};

/// A message sent on the kakoune socket starts by a 8bit message type ide (taken from kakoune src/remote.cc)
#[allow(dead_code)]
enum MessageType {
	Unknown,
	Connect,
	Command,
	MenuShow,
	MenuSelect,
	MenuHide,
	InfoShow,
	InfoHide,
	Draw,
	DrawStatus,
	SetCursor,
	Refresh,
	SetOptions,
	Exit,
	Key,
}

/// Client connection to kakoune socket
pub struct Client {
	/// command buffer
	buffer: Vec<u8>,
	/// kakoune socket path
	socket_path: String,
}

impl Client {
	/// Setup a new connection to the given session.
	pub fn new(session: &str) -> Result<Self> {
		let runtime_dir = env::var("XDG_RUNTIME_DIR");
		let socket_path = if let Ok(runtime_dir) = runtime_dir {
			format!("{}/kakoune/{}", runtime_dir, session)
		} else {
			let tmpdir = env::temp_dir();
			// TODO: get current user id using libc before falling back to env var like kakoune do
			let user = env::var("USER")?;
			format!("{}/kakoune-{}/{}", tmpdir.display(), user, session)
		};
		Ok(Self {
			// pre-allocate buffer
			buffer: vec![],
			socket_path,
		})
	}

	/// Send the commands to the client
	pub async fn send_command(&mut self, cmd: &str) -> Result<()> {
		// TODO: look a way to reuse the unix stream between send_command
		let mut stream = UnixStream::connect(format!("{}", self.socket_path))
			.await
			.context("Couldn't connect to kakoune so socket")?;
		self.buffer.clear();
		self.buffer.push(MessageType::Command as u8);
		// instead of sum, we could define a push_command and patch the size with the buffer len inside a send method
		let len = u32::try_from(cmd.len()).unwrap();
		self.buffer
			.write_u32::<NativeEndian>(len + 4 + 4 + 1)
			.unwrap();
		self.buffer.write_u32::<NativeEndian>(len).unwrap();
		std::io::Write::write(&mut self.buffer, cmd.as_bytes())?;
		stream.write_all(&self.buffer).await?;
		Ok(())
	}
}
