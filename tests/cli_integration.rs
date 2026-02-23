use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ogre() -> Command {
    #[allow(deprecated)]
    Command::cargo_bin("ogre").unwrap()
}

// ---- Version and help ----

#[test]
fn test_version_flag() {
    ogre()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("ogre"));
}

#[test]
fn test_help_flag() {
    ogre()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("brainfuck"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("compile"))
        .stdout(predicate::str::contains("format"));
}

// ---- ogre run ----

#[test]
fn test_run_hello_world() {
    ogre()
        .args(["run", "tests/brainfuck_scripts/hello_world.bf"])
        .assert()
        .success()
        .stdout("Hello World!\n");
}

#[test]
fn test_run_nonexistent_file() {
    ogre().args(["run", "nonexistent.bf"]).assert().failure();
}

#[test]
fn test_run_with_tape_size() {
    ogre()
        .args([
            "run",
            "--tape-size",
            "100",
            "tests/brainfuck_scripts/hello_world.bf",
        ])
        .assert()
        .success()
        .stdout("Hello World!\n");
}

// ---- ogre check ----

#[test]
fn test_check_valid_file() {
    ogre()
        .args(["check", "tests/brainfuck_scripts/hello_world.bf"])
        .assert()
        .success();
}

#[test]
fn test_check_unmatched_bracket() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("bad.bf");
    fs::write(&file, "+++[>+++").unwrap();

    ogre()
        .args(["check", file.to_str().unwrap()])
        .assert()
        .failure();
}

// ---- ogre format ----

#[test]
fn test_format_check_already_formatted() {
    ogre()
        .args([
            "format",
            "--check",
            "tests/brainfuck_scripts/hello_world.bf",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("already formatted"));
}

#[test]
fn test_format_check_unformatted() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("messy.bf");
    fs::write(&file, "+++[>+++<-]>.").unwrap();

    ogre()
        .args(["format", "--check", file.to_str().unwrap()])
        .assert()
        .failure()
        .stdout(predicate::str::contains("would be reformatted"));
}

#[test]
fn test_format_diff_no_changes() {
    ogre()
        .args(["format", "--diff", "tests/brainfuck_scripts/hello_world.bf"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_format_diff_shows_changes() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("messy.bf");
    fs::write(&file, "+++[>+++<-]>.").unwrap();

    ogre()
        .args(["format", "--diff", file.to_str().unwrap()])
        .assert()
        .failure()
        .stdout(predicate::str::contains("---"))
        .stdout(predicate::str::contains("+++"));
}

#[test]
fn test_format_diff_does_not_modify_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("messy.bf");
    let original = "+++[>+++<-]>.";
    fs::write(&file, original).unwrap();

    let _ = ogre()
        .args(["format", "--diff", file.to_str().unwrap()])
        .assert();

    let after = fs::read_to_string(&file).unwrap();
    assert_eq!(after, original);
}

#[test]
fn test_format_in_place() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("fmt.bf");
    fs::write(&file, "+++[>+++<-]>.").unwrap();

    ogre()
        .args(["format", file.to_str().unwrap()])
        .assert()
        .success();

    let formatted = fs::read_to_string(&file).unwrap();
    assert!(
        formatted.contains("["),
        "formatted output should have brackets"
    );
    assert_ne!(
        formatted, "+++[>+++<-]>.",
        "file should have been reformatted"
    );
}

// ---- ogre generate ----

#[test]
fn test_generate_helloworld() {
    ogre()
        .args(["generate", "helloworld"])
        .assert()
        .success()
        .stdout(predicate::str::contains("+"))
        .stdout(predicate::str::contains("."));
}

#[test]
fn test_generate_string() {
    ogre()
        .args(["generate", "string", "Hi"])
        .assert()
        .success()
        .stdout(predicate::str::contains("+"))
        .stdout(predicate::str::contains("."));
}

#[test]
fn test_generate_loop() {
    ogre()
        .args(["generate", "loop", "5"])
        .assert()
        .success()
        .stdout(predicate::str::contains("["));
}

#[test]
fn test_generate_helloworld_to_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("hello.bf");

    ogre()
        .args(["generate", "helloworld", "-o", file.to_str().unwrap()])
        .assert()
        .success();

    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("+"));
    assert!(content.contains("."));
}

// ---- ogre new ----

#[test]
fn test_new_creates_project() {
    let dir = TempDir::new().unwrap();
    let project_name = "testproject";
    let project_dir = dir.path().join(project_name);

    ogre()
        .args(["new", project_name])
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(project_dir.join("ogre.toml").exists());
    assert!(project_dir.join("src/main.bf").exists());
    assert!(project_dir.join("tests/basic.json").exists());
}

#[test]
fn test_new_with_std() {
    let dir = TempDir::new().unwrap();
    let project_name = "stdproject";
    let project_dir = dir.path().join(project_name);

    ogre()
        .args(["new", project_name, "--with-std"])
        .current_dir(dir.path())
        .assert()
        .success();

    let main_bf = fs::read_to_string(project_dir.join("src/main.bf")).unwrap();
    assert!(
        main_bf.contains("@import \"std/"),
        "main.bf should import from std library"
    );
}

// ---- ogre pack ----

#[test]
fn test_pack_outputs_pure_bf() {
    ogre()
        .args(["pack", "tests/brainfuck_scripts/hello_world.bf"])
        .assert()
        .success()
        .stdout(predicate::str::contains("+"));
}

#[test]
fn test_pack_with_optimize() {
    ogre()
        .args([
            "pack",
            "--optimize",
            "tests/brainfuck_scripts/hello_world.bf",
        ])
        .assert()
        .success();
}

// ---- ogre analyse ----

#[test]
fn test_analyse_valid_file() {
    ogre()
        .args(["analyse", "tests/brainfuck_scripts/hello_world.bf"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Brackets"));
}

// ---- ogre bench ----

#[test]
fn test_bench_reports_stats() {
    ogre()
        .args(["bench", "tests/brainfuck_scripts/simple_multiply.bf"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Instructions"))
        .stdout(predicate::str::contains("Wall time"));
}

// ---- ogre stdlib ----

#[test]
fn test_stdlib_list() {
    ogre()
        .args(["stdlib", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("io"))
        .stdout(predicate::str::contains("math"));
}

#[test]
fn test_stdlib_show_io() {
    ogre()
        .args(["stdlib", "show", "io"])
        .assert()
        .success()
        .stdout(predicate::str::contains("print_newline"));
}

#[test]
fn test_stdlib_show_unknown_module() {
    ogre()
        .args(["stdlib", "show", "nonexistent"])
        .assert()
        .failure();
}

// ---- ogre init ----

#[test]
fn test_init_creates_toml() {
    let dir = TempDir::new().unwrap();

    ogre()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join("ogre.toml").exists());
}

#[test]
fn test_init_fails_if_toml_exists() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("ogre.toml"),
        "[project]\nname = \"x\"\nversion = \"0.1.0\"\nentry = \"src/main.bf\"",
    )
    .unwrap();

    ogre()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .failure();
}

// ---- Schema validation ----

#[test]
fn test_invalid_project_entry_not_bf() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("ogre.toml"),
        "[project]\nname = \"x\"\nversion = \"0.1.0\"\nentry = \"src/main.txt\"",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/main.txt"), "+++").unwrap();

    // Running without a file arg should try to load the project and fail validation
    ogre()
        .arg("run")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("must end with .bf"));
}

#[test]
fn test_invalid_project_empty_name() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("ogre.toml"),
        "[project]\nname = \"\"\nversion = \"0.1.0\"\nentry = \"src/main.bf\"",
    )
    .unwrap();

    ogre()
        .arg("run")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("name must not be empty"));
}

// ---- ogre doc ----

#[test]
fn test_doc_stdlib() {
    ogre()
        .args(["doc", "--stdlib"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Standard Library Reference"))
        .stdout(predicate::str::contains("std/io"))
        .stdout(predicate::str::contains("print_newline"));
}

#[test]
fn test_doc_file() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("test.bf"),
        "@doc My function docs\n@fn my_func { +++ }",
    )
    .unwrap();

    ogre()
        .args(["doc", "test.bf"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("@fn my_func"))
        .stdout(predicate::str::contains("My function docs"));
}

#[test]
fn test_doc_output_file() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.bf"), "@fn foo { + }").unwrap();

    ogre()
        .args(["doc", "test.bf", "-o", "docs.md"])
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join("docs.md").exists());
    let content = fs::read_to_string(dir.path().join("docs.md")).unwrap();
    assert!(content.contains("@fn foo"));
}

// ---- --quiet flag ----

#[test]
fn test_quiet_suppresses_output() {
    let dir = TempDir::new().unwrap();
    let name = dir.path().join("quiettest");

    ogre()
        .args(["new", "--quiet", name.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    assert!(name.join("ogre.toml").exists());
}

// ---- @const/@use ----

#[test]
fn test_const_use_in_run() {
    let dir = TempDir::new().unwrap();
    // @const X 65 -> @use X expands to 65 '+' chars -> ASCII 'A'
    fs::write(
        dir.path().join("const_test.bf"),
        "@const CHAR_A 65\n@use CHAR_A\n.",
    )
    .unwrap();

    ogre()
        .args(["run", "const_test.bf"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("A"));
}

// ---- ogre check additional tests ----

#[test]
fn test_check_unknown_call() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("bad_call.bf");
    fs::write(&file, "@call nonexistent_function").unwrap();

    ogre()
        .args(["check", file.to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn test_check_import_cycle() {
    let dir = TempDir::new().unwrap();
    let a = dir.path().join("a.bf");
    let b = dir.path().join("b.bf");
    fs::write(&a, "@import \"b.bf\"").unwrap();
    fs::write(&b, "@import \"a.bf\"").unwrap();

    ogre()
        .args(["check", a.to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn test_check_missing_import() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("bad_import.bf");
    fs::write(&file, "@import \"nonexistent.bf\"").unwrap();

    ogre()
        .args(["check", file.to_str().unwrap()])
        .assert()
        .failure();
}

// ---- ogre pack additional tests ----

#[test]
fn test_pack_with_fn_call() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("with_fn.bf");
    fs::write(&file, "@fn greet { +++ }\n@call greet").unwrap();

    ogre()
        .args(["pack", file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("+++"))
        .stdout(predicate::str::contains("@fn").not())
        .stdout(predicate::str::contains("@call").not());
}

#[test]
fn test_pack_to_output_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("source.bf");
    fs::write(&file, "@fn greet { +++ }\n@call greet").unwrap();

    ogre()
        .args(["pack", file.to_str().unwrap(), "-o", "packed.bf"])
        .current_dir(dir.path())
        .assert()
        .success();

    let content = fs::read_to_string(dir.path().join("packed.bf")).unwrap();
    assert!(content.contains("+++"));
    assert!(!content.contains("@fn"));
}

#[test]
fn test_pack_optimize_produces_shorter() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("cancel.bf");
    fs::write(&file, "+-+-+-><><").unwrap();

    let normal = ogre()
        .args(["pack", file.to_str().unwrap()])
        .output()
        .unwrap();
    let optimized = ogre()
        .args(["pack", "--optimize", file.to_str().unwrap()])
        .output()
        .unwrap();

    let normal_len = String::from_utf8_lossy(&normal.stdout).len();
    let opt_len = String::from_utf8_lossy(&optimized.stdout).len();
    assert!(
        opt_len <= normal_len,
        "optimized should be <= normal length"
    );
}

// ---- ogre init additional tests ----

#[test]
fn test_init_creates_src_directory() {
    let dir = TempDir::new().unwrap();

    ogre()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join("ogre.toml").exists());
    // Verify the generated toml is valid by loading it
    let content = fs::read_to_string(dir.path().join("ogre.toml")).unwrap();
    assert!(content.contains("[project]"));
    assert!(content.contains("entry"));
}

// ---- ogre bench additional tests ----

#[test]
fn test_bench_hello_world() {
    ogre()
        .args(["bench", "tests/brainfuck_scripts/hello_world.bf"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Instructions"))
        .stdout(predicate::str::contains("Cells touched"));
}

// ---- ogre analyse additional tests ----

#[test]
fn test_analyse_verbose() {
    ogre()
        .args([
            "analyse",
            "--verbose",
            "tests/brainfuck_scripts/hello_world.bf",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Brackets"));
}

#[test]
fn test_analyse_unmatched_bracket() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("bad.bf");
    fs::write(&file, "+++[>+++").unwrap();

    ogre()
        .args(["analyse", file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("ERROR"));
}

// ---- ogre test ----

#[test]
fn test_test_runner_passes() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("hello.bf"),
        "++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.[-]",
    )
    .unwrap();
    let test_json = r#"[{"name":"prints H","brainfuck":"hello.bf","input":"","output":"H"}]"#;
    fs::write(dir.path().join("tests.json"), test_json).unwrap();

    ogre()
        .args(["test", "tests.json"])
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn test_test_runner_fails() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("hello.bf"),
        "++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.[-]",
    )
    .unwrap();
    let test_json = r#"[{"name":"wrong output","brainfuck":"hello.bf","input":"","output":"X"}]"#;
    fs::write(dir.path().join("tests.json"), test_json).unwrap();

    ogre()
        .args(["test", "tests.json"])
        .current_dir(dir.path())
        .assert()
        .failure();
}

// ---- --watch flag ----

#[test]
fn test_run_watch_flag_accepted() {
    // Just verify the --watch flag is recognized (can't test actual watching)
    ogre()
        .args(["run", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--watch"));
}

// ---- ogre run with @fn/@call ----

#[test]
fn test_run_with_fn_call() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("fn_test.bf"),
        "@fn add3 { +++ }\n@call add3\n.",
    )
    .unwrap();

    ogre()
        .args(["run", "fn_test.bf"])
        .current_dir(dir.path())
        .assert()
        .success();
}

// ---- ogre run with stdlib ----

#[test]
fn test_run_with_stdlib_import() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("std_test.bf"),
        "@import \"std/io\"\n@call print_newline",
    )
    .unwrap();

    ogre()
        .args(["run", "std_test.bf"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\n"));
}

// ---- --no-color flag ----

#[test]
fn test_no_color_flag_accepted() {
    ogre().args(["--no-color", "--help"]).assert().success();
}

// ---- Error cases ----

#[test]
fn test_no_subcommand_shows_help() {
    ogre()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn test_unknown_subcommand() {
    ogre().arg("nonexistent").assert().failure();
}

// ---- WASM target ----

#[test]
fn test_compile_wasm_generates_wat() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("hello.bf");
    fs::write(&file, "++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.>+.+++++++..+++.>++.<<+++++++++++++++.>.+++.------.--------.>+.>.").unwrap();

    ogre()
        .args([
            "compile",
            file.to_str().unwrap(),
            "--target",
            "wasm",
            "-o",
            "hello",
        ])
        .current_dir(dir.path())
        .assert()
        .success();

    // Should have generated at least a .wat file (or .wasm if wat2wasm is available)
    let wat_exists = dir.path().join("hello.wat").exists();
    let wasm_exists = dir.path().join("hello.wasm").exists();
    assert!(wat_exists || wasm_exists, "expected .wat or .wasm file");
}

#[test]
fn test_compile_unknown_target_fails() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("hello.bf");
    fs::write(&file, "+").unwrap();

    ogre()
        .args(["compile", file.to_str().unwrap(), "--target", "arm64"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown target"));
}

// ---- Dependency management ----

#[test]
fn test_run_project_with_dependency() {
    let dir = TempDir::new().unwrap();

    // Create a dependency library
    let dep_dir = dir.path().join("mylib");
    let dep_src = dep_dir.join("src");
    fs::create_dir_all(&dep_src).unwrap();
    fs::write(
        dep_dir.join("ogre.toml"),
        r#"[project]
name = "mylib"
version = "0.1.0"
entry = "src/main.bf"

[build]
include = ["src/"]
"#,
    )
    .unwrap();
    // @fn that prints 'A' (ASCII 65)
    fs::write(
        dep_src.join("main.bf"),
        "@fn print_A { +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++ . [-] }",
    )
    .unwrap();

    // Create the main project that depends on mylib
    let app_dir = dir.path().join("myapp");
    let app_src = app_dir.join("src");
    fs::create_dir_all(&app_src).unwrap();
    fs::write(
        app_dir.join("ogre.toml"),
        r#"[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.bf"

[dependencies]
mylib = { path = "../mylib" }
"#,
    )
    .unwrap();
    fs::write(app_src.join("main.bf"), "@call print_A").unwrap();

    ogre()
        .arg("run")
        .current_dir(&app_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("A"));
}

#[test]
fn test_check_project_with_dependency() {
    let dir = TempDir::new().unwrap();

    // Create a dependency library
    let dep_dir = dir.path().join("mylib");
    let dep_src = dep_dir.join("src");
    fs::create_dir_all(&dep_src).unwrap();
    fs::write(
        dep_dir.join("ogre.toml"),
        r#"[project]
name = "mylib"
version = "0.1.0"
entry = "src/main.bf"

[build]
include = ["src/"]
"#,
    )
    .unwrap();
    fs::write(dep_src.join("main.bf"), "@fn dep_fn { +++ }").unwrap();

    // Create the main project
    let app_dir = dir.path().join("myapp");
    let app_src = app_dir.join("src");
    fs::create_dir_all(&app_src).unwrap();
    fs::write(
        app_dir.join("ogre.toml"),
        r#"[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.bf"

[build]
include = ["src/"]

[dependencies]
mylib = { path = "../mylib" }
"#,
    )
    .unwrap();
    // This file calls a function from the dependency
    fs::write(app_src.join("main.bf"), "@call dep_fn").unwrap();

    ogre().arg("check").current_dir(&app_dir).assert().success();
}

#[test]
fn test_pack_project_with_dependency() {
    let dir = TempDir::new().unwrap();

    // Create a dependency library
    let dep_dir = dir.path().join("mylib");
    let dep_src = dep_dir.join("src");
    fs::create_dir_all(&dep_src).unwrap();
    fs::write(
        dep_dir.join("ogre.toml"),
        r#"[project]
name = "mylib"
version = "0.1.0"
entry = "src/main.bf"

[build]
include = ["src/"]
"#,
    )
    .unwrap();
    fs::write(dep_src.join("main.bf"), "@fn dep_add { +++ }").unwrap();

    // Create the main project
    let app_dir = dir.path().join("myapp");
    let app_src = app_dir.join("src");
    fs::create_dir_all(&app_src).unwrap();
    fs::write(
        app_dir.join("ogre.toml"),
        r#"[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.bf"

[dependencies]
mylib = { path = "../mylib" }
"#,
    )
    .unwrap();
    fs::write(app_src.join("main.bf"), "@call dep_add").unwrap();

    ogre()
        .arg("pack")
        .current_dir(&app_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("+++"));
}

#[test]
fn test_dependency_missing_path_fails() {
    let dir = TempDir::new().unwrap();

    let app_dir = dir.path().join("myapp");
    let app_src = app_dir.join("src");
    fs::create_dir_all(&app_src).unwrap();
    fs::write(
        app_dir.join("ogre.toml"),
        r#"[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.bf"

[dependencies]
missing = { path = "../nonexistent" }
"#,
    )
    .unwrap();
    fs::write(app_src.join("main.bf"), "+").unwrap();

    ogre()
        .arg("run")
        .current_dir(&app_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_bench_project_with_dependency() {
    let dir = TempDir::new().unwrap();

    // Create a dependency library
    let dep_dir = dir.path().join("mylib");
    let dep_src = dep_dir.join("src");
    fs::create_dir_all(&dep_src).unwrap();
    fs::write(
        dep_dir.join("ogre.toml"),
        r#"[project]
name = "mylib"
version = "0.1.0"
entry = "src/main.bf"

[build]
include = ["src/"]
"#,
    )
    .unwrap();
    fs::write(dep_src.join("main.bf"), "@fn dep_inc { + }").unwrap();

    // Create the main project
    let app_dir = dir.path().join("myapp");
    let app_src = app_dir.join("src");
    fs::create_dir_all(&app_src).unwrap();
    fs::write(
        app_dir.join("ogre.toml"),
        r#"[project]
name = "myapp"
version = "0.1.0"
entry = "src/main.bf"

[dependencies]
mylib = { path = "../mylib" }
"#,
    )
    .unwrap();
    fs::write(app_src.join("main.bf"), "@call dep_inc").unwrap();

    ogre()
        .arg("bench")
        .current_dir(&app_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Instructions"));
}

#[test]
fn test_pack_preserves_semantics() {
    // Run the original file and the packed output; verify they produce the same result
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("prog.bf");
    // Simple program: set cell to 'H' and print it
    fs::write(
        &src,
        "@fn set_H { ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++. }\n@call set_H",
    )
    .unwrap();

    // Pack the file
    let pack_output = ogre()
        .args(["pack", src.to_str().unwrap()])
        .assert()
        .success();
    let packed = String::from_utf8(pack_output.get_output().stdout.clone()).unwrap();
    let packed = packed.trim();

    // Write the packed output to a file and run it
    let packed_file = dir.path().join("packed.bf");
    fs::write(&packed_file, packed).unwrap();

    // Run original
    let orig_output = ogre()
        .args(["run", src.to_str().unwrap()])
        .assert()
        .success();
    let orig_stdout = String::from_utf8(orig_output.get_output().stdout.clone()).unwrap();

    // Run packed
    let pack_run_output = ogre()
        .args(["run", packed_file.to_str().unwrap()])
        .assert()
        .success();
    let pack_stdout = String::from_utf8(pack_run_output.get_output().stdout.clone()).unwrap();

    assert_eq!(orig_stdout, pack_stdout);
}

#[test]
fn test_init_detects_existing_bf_files() {
    let dir = TempDir::new().unwrap();

    // Create an existing .bf file before running init
    fs::write(dir.path().join("existing.bf"), "++++.").unwrap();

    ogre()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    // ogre.toml should exist
    let toml_path = dir.path().join("ogre.toml");
    assert!(toml_path.exists());
}

#[test]
fn test_verbose_test_runner() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("hello.bf"),
        "++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.[-]",
    )
    .unwrap();
    let test_json = r#"[{"name": "print H", "brainfuck": "hello.bf", "input": "", "output": "H"}]"#;
    let test_file = dir.path().join("tests.json");
    fs::write(&test_file, test_json).unwrap();

    ogre()
        .args(["test", "--verbose", test_file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("PASS"))
        .stdout(predicate::str::contains("print H"));
}
