use rand::{
	distributions::Alphanumeric,
	{thread_rng, Rng},
};

/// return random 10 chars string
pub fn mktemp(len: usize) -> String {
	thread_rng()
		.sample_iter(&Alphanumeric)
		.take(len)
		.map(char::from)
		.collect()
}
