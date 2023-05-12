use std::path::PathBuf;

use git_cliff as cliff;
use git_cliff::args::Sort::Oldest;

#[test]
#[ignore]
fn generate_changelog() {
    cliff::run(cliff::args::Opt {
        help: None,
        version: None,
        verbose: 0,
        config: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cliff.toml"),
        workdir: None,
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
        unreleased: false,
        topo_order: false,
        context: false,
        strip: None,
        sort: Oldest,
        range: None,
    })
    .expect("Couldn't generate Changelog");
}
