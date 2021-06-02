use kak::{face, escape::Mode};

use anyhow::Result;
use async_std::io;

/// Forward stdin to stdout after converting all ansi color code to kakoune face definition
pub async fn faces() -> Result<()> {
	let stdin = io::stdin();
	let mut line = String::new();
	while let Ok(size) = stdin.read_line(&mut line).await {
		if size == 0 {
			break;
		}
		face::print(&line, Mode::Brace);
		line.clear();
	}
	Ok(())
}
