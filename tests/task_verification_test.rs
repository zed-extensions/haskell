// Heavily inspired by the Java extension's task verification tests:
// https://github.com/zed-extensions/java/blob/b6b8f9a5396b7f794d12a91ee4c703078ecc3e96/tests/task_verification_test.rs

use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn get_task_command_by_tag(tag: &str) -> String {
    let tasks_json =
        fs::read_to_string("languages/haskell/tasks.json").expect("Failed to read tasks.json");
    let tasks: Value = serde_json::from_str(&tasks_json).expect("Failed to parse tasks.json");
    for task in tasks.as_array().expect("tasks.json is not an array") {
        if let Some(tags) = task["tags"].as_array() {
            if tags.iter().any(|t| t.as_str() == Some(tag)) {
                return task["command"]
                    .as_str()
                    .expect("command is not a string")
                    .to_string();
            }
        }
    }
    panic!("Task with tag '{tag}' not found");
}

struct TestProject {
    temp_dir: PathBuf,
    bin_dir: PathBuf,
    new_path: String,
}

impl TestProject {
    fn new(name: &str) -> Self {
        let temp_dir = std::env::temp_dir().join(format!("haskell_task_test_{name}"));
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();
        let bin_dir = temp_dir.join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let new_path = format!(
            "{}:{}",
            bin_dir.to_string_lossy(),
            std::env::var("PATH").unwrap_or_default()
        );
        Self {
            temp_dir,
            bin_dir,
            new_path,
        }
    }

    fn with_stack(&self) {
        fs::File::create(self.temp_dir.join("stack.yaml")).unwrap();
        fs::File::create(self.temp_dir.join("myproject.cabal")).unwrap();
        self.mock_bin("stack", "#!/bin/sh\necho \"STACK_CALLED: $@\"");
    }

    fn with_cabal(&self) {
        fs::File::create(self.temp_dir.join("myproject.cabal")).unwrap();
        self.mock_bin("cabal", "#!/bin/sh\necho \"CABAL_CALLED: $@\"");
    }

    fn with_cabal_project(&self) {
        fs::File::create(self.temp_dir.join("cabal.project")).unwrap();
        self.mock_bin("cabal", "#!/bin/sh\necho \"CABAL_CALLED: $@\"");
    }

    fn mock_bin(&self, name: &str, content: &str) {
        let bin_path = self.bin_dir.join(name);
        fs::write(&bin_path, content).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&bin_path, fs::Permissions::from_mode(0o755)).unwrap();
        }
    }

    fn run_task(&self, tag: &str) -> String {
        let command = get_task_command_by_tag(tag);
        let output = Command::new("sh")
            .arg("-c")
            .arg(&command)
            .env("PATH", &self.new_path)
            .current_dir(&self.temp_dir)
            .output()
            .expect("Failed to execute shell command");
        String::from_utf8_lossy(&output.stdout).into_owned()
            + &String::from_utf8_lossy(&output.stderr)
    }
}

impl Drop for TestProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

#[test]
fn test_stack_build() {
    let project = TestProject::new("stack_build");
    project.with_stack();
    let output = project.run_task("haskell-build");
    assert!(output.contains("STACK_CALLED: build"), "Got: {}", output);
}

#[test]
fn test_stack_run() {
    let project = TestProject::new("stack_run");
    project.with_stack();
    let output = project.run_task("haskell-run");
    assert!(output.contains("STACK_CALLED: run"), "Got: {}", output);
}

#[test]
fn test_cabal_build() {
    let project = TestProject::new("cabal_build");
    project.with_cabal();
    let output = project.run_task("haskell-build");
    assert!(output.contains("CABAL_CALLED: build"), "Got: {}", output);
}

#[test]
fn test_cabal_run() {
    let project = TestProject::new("cabal_run");
    project.with_cabal();
    let output = project.run_task("haskell-run");
    assert!(output.contains("CABAL_CALLED: run"), "Got: {}", output);
}

#[test]
fn test_cabal_project_build() {
    let project = TestProject::new("cabal_project_build");
    project.with_cabal_project();
    let output = project.run_task("haskell-build");
    assert!(output.contains("CABAL_CALLED: build"), "Got: {}", output);
}

#[test]
fn test_cabal_project_run() {
    let project = TestProject::new("cabal_project_run");
    project.with_cabal_project();
    let output = project.run_task("haskell-run");
    assert!(output.contains("CABAL_CALLED: run"), "Got: {}", output);
}

#[test]
fn test_stack_preferred_over_cabal() {
    let project = TestProject::new("stack_preferred");
    project.with_stack();
    project.with_cabal();
    let output = project.run_task("haskell-run");
    assert!(
        output.contains("STACK_CALLED: run"),
        "Stack should be preferred when stack.yaml present. Got: {}",
        output
    );
}

#[test]
fn test_no_build_tool() {
    let project = TestProject::new("no_tool");
    let output = project.run_task("haskell-build");
    assert!(
        output.contains("No Stack or Cabal found"),
        "Got: {}",
        output
    );
}
