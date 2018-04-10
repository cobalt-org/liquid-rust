extern crate glob;
extern crate which;

#[test]
fn readme_test() {
    let rustdoc = which::which("rustdoc").unwrap();

    let readme = std::path::Path::new(file!()).canonicalize().unwrap();
    let readme = readme.parent().unwrap().parent().unwrap().join("README.md");
    let readme = readme.to_str().unwrap();

    let deps = std::path::Path::new(&std::env::current_exe().unwrap())
        .canonicalize()
        .unwrap();
    let deps = deps.parent().unwrap();

    let mut cmd = std::process::Command::new(rustdoc);
    cmd.arg("--verbose")
        .args(&["--library-path", deps.to_str().unwrap()])
        .arg("--test")
        .arg(&readme);

    let result = cmd.spawn()
        .expect("Failed to spawn rustdoc process")
        .wait()
        .expect("Failed to run rustdoc process");

    assert!(result.success(), "Failed to run rustdoc tests on README.md");
}
