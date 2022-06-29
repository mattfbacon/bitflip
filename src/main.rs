use std::fs::File;
use std::io::{Read as _, Seek as _, SeekFrom, Write as _};
use std::path::PathBuf;

use anyhow::Context as _;

#[derive(argh::FromArgs)]
/// flip bits in a file
struct Args {
	/// the number of bits to flip
	#[argh(option)]
	num_bits: usize,
	/// the file to flip bits in
	#[argh(positional)]
	file: PathBuf,
}

fn main() -> anyhow::Result<()> {
	let args: Args = argh::from_env();
	let mut file = File::options()
		.read(true)
		.write(true)
		.truncate(false)
		.create(false)
		.open(args.file)
		.context("opening file")?;
	let file_len = file.metadata().context("reading file metadata")?.len();
	let num_bits_in_file = file_len
		.checked_mul(8)
		.and_then(|bits| usize::try_from(bits).ok())
		.context("file is too large to be manipulated")?;
	let indexes = rand::seq::index::sample(&mut rand::thread_rng(), num_bits_in_file, args.num_bits);
	for index in indexes {
		let byte_index = u64::try_from(index / 8).unwrap();
		let bit_in_byte = u8::try_from(index % 8).unwrap();
		file
			.seek(SeekFrom::Start(byte_index))
			.context("seeking to byte in file")?;
		let mut byte_at_index = {
			let mut buf = [0u8; 1];
			file
				.read_exact(&mut buf)
				.context("reading byte from file")?;
			buf[0]
		};
		byte_at_index ^= 1 << bit_in_byte;
		file
			.seek(SeekFrom::Start(byte_index))
			.context("seeking to byte in file")?;
		file
			.write_all(&[byte_at_index])
			.context("writing byte back to file")?;
	}
	Ok(())
}
