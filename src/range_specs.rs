use std::collections::VecDeque;

use kak::range::Range;

pub struct SharedRanges {
	pub ranges: VecDeque<Range>,
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
