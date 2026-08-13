use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn every_basic_and_algorithm_example_runs() {
    let examples_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut programs = peps_files(&examples_root);
    programs.sort();

    assert!(
        !programs.is_empty(),
        "the examples directory should contain programs"
    );
    for program in programs {
        let source = fs::read_to_string(&program)
            .unwrap_or_else(|error| panic!("could not read {}: {error}", program.display()));
        let inputs = match program.file_name().and_then(|name| name.to_str()) {
            Some("09-input.peps") => vec!["hello Peps", "41", "3.5", "true"],
            Some("overview.peps") => vec!["42"],
            _ => Vec::new(),
        };
        peps::run_source_with_inputs(&source, inputs).unwrap_or_else(|error| {
            panic!("{} failed: {:?}", program.display(), error.diagnostics)
        });
    }
}

fn peps_files(directory: &Path) -> Vec<PathBuf> {
    let entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("could not read {}: {error}", directory.display()));
    let mut programs = Vec::new();

    for entry in entries {
        let path = entry
            .expect("example directory entry should be readable")
            .path();
        if path.is_dir() {
            programs.extend(peps_files(&path));
        } else if path
            .extension()
            .is_some_and(|extension| extension == "peps")
        {
            programs.push(path);
        }
    }

    programs
}
