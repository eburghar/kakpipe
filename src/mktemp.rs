use rand::{
	distributions::Alphanumeric,
	{thread_rng, Rng},
};
use std::{env, fs::create_dir_all, path::{Path, PathBuf}};
use anyhow::{bail, Result};

/// return random 10 chars string
pub fn temp_id(len: usize) -> String {
	thread_rng()
		.sample_iter(&Alphanumeric)
		.take(len)
		.map(char::from)
		.collect()
}

/// return path buf from a path, name and an extension
pub fn temp_file(path: &Path, name: &str, ext: &str) -> Result<PathBuf> {
	let mut res = path.join(name);
	if res.set_extension(ext) {
    	Ok(res)
    } else {
        bail!("Failed to create {:?}/{}.{}", path, name, ext)
    }
}

pub fn temp_dir(basename: &str) -> Result<PathBuf> {
	let tmp_dir = env::temp_dir().join(basename);
	create_dir_all(&tmp_dir)?;
	Ok(tmp_dir)
}
