use std::collections::VecDeque;

use kak::range::Range;

/// Shared data between `stdin_fifo` which read stdout and stderr from
/// the spawned command and extract ranges from ansi-code and `range_specs`
/// which consumes the ranges
pub struct SharedRanges {
	// a fifo queue of ranges
	pub ranges: VecDeque<Range>,
	// marker to signal `range_specs` that fifo has been closed
	pub fifo_end: bool,
}

impl SharedRanges {
	pub fn new() -> Self {
		Self {
			ranges: VecDeque::new(),
			fifo_end: false,
		}
	}
}
