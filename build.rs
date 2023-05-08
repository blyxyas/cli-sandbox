fn main() {
	#[cfg(all(feature = "dev", feature = "release"))]
	#[rustfmt::skip]
	compile_error!("You cannot have both `dev` and `release` features enabled at the same time.");
	#[cfg(all(not(feature = "dev"), not(feature = "release")))]
	compile_error!("You must enable either `dev` or `release` to use this crate. (Not at the same time, `dev` is recommended)")
}