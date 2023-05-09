use std::path::PathBuf;

use git_cliff as cliff;

#[test]
#[ignore]
fn generate_changelog() {
	cliff::run(cliff::args::Opt {
        help: None,
        version: None,
        verbose: 0,
        config: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cliff.toml"),
        workdir: Some(PathBuf::from(env!("CARGO_MANIFEST_DIR"))),
        repository: None,
        include_path: None,
        exclude_path: None,
        with_commit: None,
        prepend: None,
        output: Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("CHANGELOG.md")),
        tag: None,
        body: None,
        init: false,
        latest: false,
        current: false,
        unreleased: true,
        topo_order: false,
        context: false,
        strip: None,
        sort: cliff::args::Sort::Newest,
        range: None,
    })
    .expect("Hi");
}
