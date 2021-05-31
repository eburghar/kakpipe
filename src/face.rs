//! Convert ansi-code to kakoune face

use crate::escape::{EscapeIterator, Token, Mode};

use yew_ansi::{get_sgr_segments, ColorEffect, SgrEffect};

/// Return the kakoune face representation of a parsed ansi-code into `face` String
pub fn display_face(effect: &SgrEffect, mut face: String) -> String {
	let has_fg = effect.fg != ColorEffect::None;
	let has_bg = effect.bg != ColorEffect::None;
	let has_option =
		effect.italic || effect.underline || effect.bold || effect.reverse || effect.dim;

	face.clear();

	if has_fg || has_bg || has_option {
		match effect.fg {
			ColorEffect::Name(color) => face.push_str(&format!("{}", color)),
			ColorEffect::NameBright(color) => face.push_str(&format!("bright-{}", color)),
			ColorEffect::Rgb(color) => face.push_str(&format!("rgb:{:X}", color)),
			ColorEffect::None => {
				if has_bg {
					face.push_str("default")
				}
			}
		};

		if has_fg && has_bg {
			face.push(',');
		}

		match effect.bg {
			ColorEffect::Name(color) => face.push_str(&format!("{}", color)),
			ColorEffect::NameBright(color) => face.push_str(&format!("bright-{}", color)),
			ColorEffect::Rgb(color) => face.push_str(&format!("rgb:{:X}", color)),
			ColorEffect::None => (),
		};

		if has_option {
			face.push('+');
			if effect.italic {
				face.push('i');
			}
			if effect.underline {
				face.push('u');
			}
			if effect.bold {
				face.push('b');
			}
			if effect.reverse {
				face.push('r');
			}
			if effect.dim {
				face.push('d');
			}
		}
	}

	face
}

/// print a text and replace ansi-code with kakoune faces {\[bgcolor\]\[,fgcolor\]\[+options\]}
pub fn print(ansi: &str) {
	let mut face = String::with_capacity(64);
	for (effect, txt) in get_sgr_segments(ansi) {
		face = display_face(&effect, face);
		if face != "" {
			print!("{{{}}}", face);
		}
		for token in EscapeIterator::new(txt, Mode::Brace) {
			match token {
				Token::OpenBrace => print!("\\{{"),
				Token::Str(txt) => print!("{}", txt),
				_ => ()
			}
		}
	}
}
