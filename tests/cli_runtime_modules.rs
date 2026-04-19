use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(prefix: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{}_{}", prefix, unique));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn run_handles_nested_local_module_imports_at_runtime() {
    let temp_dir = TestDir::new("nevermind_cli_nested_modules");
    let pkg_dir = temp_dir.path.join("pkg");
    fs::create_dir_all(&pkg_dir).unwrap();

    fs::write(
        temp_dir.path.join("main.nm"),
        "from \"pkg/foo\" import read_value\n\nfn main() do\n  print read_value()\nend\n",
    )
    .unwrap();
    fs::write(
        pkg_dir.join("foo.nm"),
        "from \"bar\" import value\n\nexport fn read_value() do\n  value\nend\n",
    )
    .unwrap();
    fs::write(pkg_dir.join("bar.nm"), "export let value = 42\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_nevermind"))
        .current_dir(&temp_dir.path)
        .arg("run")
        .arg("main.nm")
        .output()
        .unwrap();

    if !output.status.success() {
        panic!(
            "stdout:\n{}\n\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("42"));

    let foo_python = fs::read_to_string(pkg_dir.join("foo.py")).unwrap();
    assert!(foo_python.contains("from pkg.bar import value"));
}

#[test]
fn run_handles_nested_local_namespace_imports_at_runtime() {
    let temp_dir = TestDir::new("nevermind_cli_nested_namespace_modules");
    let pkg_dir = temp_dir.path.join("pkg");
    fs::create_dir_all(&pkg_dir).unwrap();

    fs::write(
        temp_dir.path.join("main.nm"),
        "from \"pkg/foo\" import read_value\n\nfn main() do\n  print read_value()\nend\n",
    )
    .unwrap();
    fs::write(
        pkg_dir.join("foo.nm"),
        "use \"bar\"\n\nexport fn read_value() do\n  bar.value\nend\n",
    )
    .unwrap();
    fs::write(pkg_dir.join("bar.nm"), "export let value = 7\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_nevermind"))
        .current_dir(&temp_dir.path)
        .arg("run")
        .arg("main.nm")
        .output()
        .unwrap();

    if !output.status.success() {
        panic!(
            "stdout:\n{}\n\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("7"));

    let foo_python = fs::read_to_string(pkg_dir.join("foo.py")).unwrap();
    assert!(foo_python.contains("import pkg.bar as bar"));
}

#[test]
fn nested_local_imports_prefer_the_importing_directory() {
    let temp_dir = TestDir::new("nevermind_cli_nested_module_precedence");
    let pkg_dir = temp_dir.path.join("pkg");
    fs::create_dir_all(&pkg_dir).unwrap();

    fs::write(
        temp_dir.path.join("main.nm"),
        "from \"pkg/foo\" import read_value\n\nfn main() do\n  print read_value()\nend\n",
    )
    .unwrap();
    fs::write(temp_dir.path.join("bar.nm"), "export let value = 1\n").unwrap();
    fs::write(
        pkg_dir.join("foo.nm"),
        "from \"bar\" import value\n\nexport fn read_value() do\n  value\nend\n",
    )
    .unwrap();
    fs::write(pkg_dir.join("bar.nm"), "export let value = 42\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_nevermind"))
        .current_dir(&temp_dir.path)
        .arg("run")
        .arg("main.nm")
        .output()
        .unwrap();

    if !output.status.success() {
        panic!(
            "stdout:\n{}\n\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("42"));
    assert!(!stdout.contains("\n1\n"));

    let foo_python = fs::read_to_string(pkg_dir.join("foo.py")).unwrap();
    assert!(foo_python.contains("from pkg.bar import value"));
}
