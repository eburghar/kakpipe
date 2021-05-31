use rand::{
	distributions::Alphanumeric,
	{thread_rng, Rng},
};

pub fn mktemp(ext: &str) -> String {
	let mut temp_name: String = thread_rng()
		.sample_iter(&Alphanumeric)
		.take(10)
		.map(char::from)
		.collect();
	temp_name.push_str(ext);
	temp_name
}
