use better_panic::{Settings, Verbosity};
use blake2::{Blake2s256, Digest};
use cargo_metadata::Package;
use file_hashing::get_hash_folder;
use git2::{self, Repository};
use semver;
use std::{env, fs, str};

#[test]
#[ignore]
fn version() {
    Settings::new().verbosity(Verbosity::Minimal).install();
    let bless = match env::var("BLESS").unwrap_or(String::from("false")).as_str() {
        "true" => true,
        _ => false,
    };
    let repo = Repository::open(".").expect("Couldn't open git repository");

    let mut hash = Blake2s256::new();
    let src_hash = get_hash_folder("src", &mut hash, num_cpus::get(), |_| {}).unwrap();

    let md = cargo_metadata::MetadataCommand::new()
        .exec()
        .expect("Couldn't get cargo metadata");

    let mut package = None;
    for mdpackage in md.packages {
        if mdpackage.name == "cli-sandbox" {
            package = Some(mdpackage);
        }
    }

    let package = match package {
        Some(p) => p,
        None => panic!("Couldn't get package `cli-sandbox` (bug)"),
    };

    // If src is changed
    if src_hash != fs::read_to_string("src.hash").expect("Couldn't read src.hash") {
        if bless {
            fs::write("src.hash", src_hash).expect("Couldn't write to src.hash");
            return;
        }
        repo.tag_foreach(|_, name| {
			let tag = &str::from_utf8(name).expect("Tag isn't valid utf-8 (somehow)")[11..];

			if semver::Version::parse(tag).expect("Couldn't parse tag to semver (bug)")
            >= package.version
			{
				panic!("The package is outdated, change it in Cargo.toml, and then run this same command with BLESS=true");
			}

			true
		})
		.expect("Couldn't loop between tags");
    }
}
