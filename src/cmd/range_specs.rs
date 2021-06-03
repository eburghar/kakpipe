use crate::args::RangeSpecsArgs;

use anyhow::{Context, Result};
use async_std::{
	io::prelude::{ReadExt, WriteExt},
	os::unix::net::UnixStream,
};
use std::convert::TryFrom;

/// Connects to the given unix socket, reads the available range_specs up to range given
/// as parameter and return the kakoune command for setting the ranges in the buffer
pub async fn range_specs(args: RangeSpecsArgs) -> Result<()> {
	let mut stream = UnixStream::connect(args.socket)
		.await
		.context("Couldn't connect to kakpipe socket")?;

	// send the range of the text to highlight to the server as a string i1.j1,i2.j2
	let len = args.range.len();
	let mut buffer = Vec::<u8>::with_capacity(len + 1);
	buffer.push(u8::try_from(len).unwrap());
	buffer.extend(args.range.as_bytes());
	stream.write_all(&buffer).await?;

	// wait for the response
	let mut response = String::new();
	let size = stream.read_to_string(&mut response).await?;

	if size != 0 {
		println!(
			"update-option buffer kakpipe_color_ranges\n\
			set-option -add buffer kakpipe_color_ranges {}",
			response
		);
	} else {
		println!("nop");
	}
	Ok(())
}
