use std::{
	collections::VecDeque,
	fmt, num,
	str::FromStr,
};
use yew_ansi::{ColorEffect, SgrEffect};

use kakpipe::face;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Pos(pub u32, pub u32);

impl Pos {
	pub fn is_valid(&self) -> bool {
		self.0 != 0 && self.1 != 0
	}
}

impl FromStr for Pos {
	type Err = num::ParseIntError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let coords: Vec<Option<u32>> = s.split('.').map(|s| s.parse::<u32>().ok()).collect();
		Ok(Pos(
			coords.get(0).unwrap_or(&Some(0u32)).unwrap(),
			coords.get(1).unwrap_or(&Some(0u32)).unwrap(),
		))
	}
}

impl fmt::Display for Pos {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}.{}", self.0, self.1)
	}
}

#[derive(Copy, Clone)]
pub struct Selection(pub Pos, pub Pos);

impl Selection {
	pub fn is_valid(&self) -> bool {
		self.0.is_valid() && self.1.is_valid() && self.0 < self.1
	}
}

impl FromStr for Selection {
	type Err = num::ParseIntError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let pos: Vec<Option<Pos>> = s.split(',').map(|s| s.parse::<Pos>().ok()).collect();
		Ok(Selection(
			pos.get(0).unwrap_or(&Some(Pos(0, 0))).unwrap(),
			pos.get(1).unwrap_or(&Some(Pos(0, 0))).unwrap(),
		))
	}
}

impl fmt::Display for Selection {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{},{}", self.0, self.1)
	}
}

pub struct Range {
	pub selection: Selection,
	pub face: SgrEffect,
}

impl Range {
	pub fn new(selection: Selection, face: SgrEffect) -> Option<Self> {
		if face.fg != ColorEffect::None
			|| face.bg != ColorEffect::None
			|| face.bold
			|| face.dim
			|| face.italic
			|| face.underline
			|| face.reverse
		{
			Some(Self { selection, face })
		} else {
			None
		}
	}
}

impl fmt::Display for Range {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut face = String::with_capacity(64);
		face = face::display_face(&self.face, face);
		write!(f, "{}|{}", self.selection, face)
	}
}

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

#[cfg(test)]
use std::cmp::Ordering;

#[test]
fn pos_less() {
	let pos1 = Pos(1, 2);
	let pos2 = Pos(1, 4);
	assert_eq!(pos1.cmp(&pos2), Ordering::Less);
}

#[test]
fn pos_greater() {
	let pos1 = Pos(2, 2);
	let pos2 = Pos(1, 4);
	assert_eq!(pos1.cmp(&pos2), Ordering::Greater);
}

#[test]
fn pos_equal() {
	let pos1 = Pos(1, 2);
	let pos2 = Pos(1, 2);
	assert_eq!(pos1.cmp(&pos2), Ordering::Equal);
}
