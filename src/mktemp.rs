use rand::{
	distributions::Alphanumeric,
	{thread_rng, Rng},
};

/// return random 10 chars string
pub fn mktemp(len: usize, ext: &str) -> String {
	let mut temp_name: String = thread_rng()
		.sample_iter(&Alphanumeric)
		.take(len)
		.map(char::from)
		.collect();
	temp_name.push_str(ext);
	temp_name
}
