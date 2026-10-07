use anyhow::Result;
use git2::{Repository, Signature, Time};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_path(prefix: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    std::env::temp_dir().join(format!(
        "{prefix}-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    ))
}

struct TempHome {
    path: PathBuf,
}

struct TestRepo {
    path: PathBuf,
    repo: Repository,
}

impl TempHome {
    fn new() -> Result<Self> {
        let path = unique_path("gitlogue-cli-home");
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    fn config_path(&self) -> PathBuf {
        self.path.join(".config/gitlogue/config.toml")
    }
}

impl TestRepo {
    fn new() -> Result<Self> {
        let path = unique_path("gitlogue-cli-repo");
        fs::create_dir_all(&path)?;
        let repo = Repository::init(&path)?;
        Ok(Self { path, repo })
    }

    fn write_file(&self, relative_path: &str, content: &str) -> Result<()> {
        let file_path = self.path.join(relative_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, content)?;
        Ok(())
    }

    fn stage_file(&self, relative_path: &str) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_path(Path::new(relative_path))?;
        index.write()?;
        Ok(())
    }

    fn commit_file(
        &self,
        relative_path: &str,
        content: &str,
        message: &str,
        timestamp: i64,
    ) -> Result<String> {
        self.write_file(relative_path, content)?;
        self.stage_file(relative_path)?;

        let mut index = self.repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        let signature = Signature::new("Test User", "test@example.com", &Time::new(timestamp, 0))?;
        let parent = self
            .repo
            .head()
            .ok()
            .and_then(|head| head.peel_to_commit().ok());
        let oid = match parent.as_ref() {
            Some(parent_commit) => self.repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &[parent_commit],
            )?,
            None => self
                .repo
                .commit(Some("HEAD"), &signature, &signature, message, &tree, &[])?,
        };

        Ok(oid.to_string())
    }
}

impl Drop for TempHome {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

impl Drop for TestRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn gitlogue_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_gitlogue"))
}

fn command_with_home(home: &TempHome) -> Command {
    let mut command = gitlogue_command();
    command
        .env("HOME", &home.path)
        .env("USERPROFILE", &home.path)
        .env_remove("HOMEDRIVE")
        .env_remove("HOMEPATH");
    command
}

fn run_command(command: &mut Command) -> Result<Output> {
    Ok(command.output()?)
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn run_in_pseudo_tty(home: &TempHome, args: &[&str], input: &[u8]) -> Result<Output> {
    let command = std::iter::once(shell_quote(env!("CARGO_BIN_EXE_gitlogue")))
        .chain(args.iter().map(|arg| shell_quote(arg)))
        .collect::<Vec<_>>()
        .join(" ");
    let mut child = Command::new("script")
        .args(["-qefc", &command, "/dev/null"])
        .env("HOME", &home.path)
        .env("USERPROFILE", &home.path)
        .env_remove("HOMEDRIVE")
        .env_remove("HOMEPATH")
        .env("TERM", "xterm-256color")
        .env("COLUMNS", "100")
        .env("LINES", "40")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .as_mut()
        .expect("script stdin should be piped")
        .write_all(input)?;
    let _ = child.stdin.take();
    Ok(child.wait_with_output()?)
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

fn repo_path(repo: &TestRepo) -> &Path {
    repo.path.as_path()
}

#[test]
fn license_flag_prints_third_party_licenses() -> Result<()> {
    let output = run_command(gitlogue_command().arg("--license"))?;
    assert!(output.status.success());
    let stdout = stdout(&output);

    assert!(stdout.starts_with(include_str!("../LICENSE-THIRD-PARTY")));

    Ok(())
}

#[test]
fn theme_subcommands_list_and_set_default_theme() -> Result<()> {
    let home = TempHome::new()?;

    let list_output = run_command(command_with_home(&home).args(["theme", "list"]))?;
    assert!(list_output.status.success());
    let list_stdout = stdout(&list_output);
    assert!(list_stdout.contains("Available themes:"));
    assert!(list_stdout.contains("  - nord"));

    let set_output = run_command(command_with_home(&home).args(["theme", "set", "nord"]))?;
    assert!(set_output.status.success());
    let set_stdout = stdout(&set_output);
    let config = fs::read_to_string(home.config_path())?;

    assert!(set_stdout.contains("Theme set to 'nord'"));
    assert!(config.contains("theme = \"nord\""));

    Ok(())
}

#[test]
fn theme_set_rejects_unknown_theme_without_writing_config() -> Result<()> {
    let home = TempHome::new()?;

    let output = run_command(command_with_home(&home).args(["theme", "set", "missing-theme"]))?;

    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("Unknown theme: missing-theme"));
    assert!(stderr(&output).contains("Available themes:"));
    assert!(!home.config_path().exists());

    Ok(())
}

#[test]
fn diff_subcommand_reports_no_changes_for_clean_repo() -> Result<()> {
    let repo = TestRepo::new()?;

    let output = run_command(gitlogue_command().args([
        "--path",
        repo_path(&repo).to_str().unwrap(),
        "diff",
    ]))?;
    assert!(output.status.success());

    assert_eq!(stdout(&output), "No changes to display\n");

    Ok(())
}

#[test]
fn invalid_ignore_pattern_is_reported_before_starting_playback() -> Result<()> {
    let home = TempHome::new()?;
    let repo = TestRepo::new()?;
    repo.commit_file("src/lib.rs", "fn clean() {}\n", "initial", 1)?;
    let path = repo_path(&repo).to_str().unwrap();

    for mode_args in [
        vec!["diff", "--ignore", "[invalid"],
        vec!["--ignore", "[invalid", "--commit", "HEAD"],
        vec!["--ignore", "[invalid", "--watch"],
    ] {
        let mut command = command_with_home(&home);
        command
            .args(["--path", path])
            .args(mode_args.iter().copied());
        let output = run_command(&mut command)?;

        assert!(
            !output.status.success(),
            "args {mode_args:?} unexpectedly succeeded; stdout: {}; stderr: {}",
            stdout(&output),
            stderr(&output)
        );
        assert_eq!(stdout(&output), "");
        assert!(
            stderr(&output).contains("Invalid glob pattern: [invalid"),
            "unexpected stderr for args {mode_args:?}: {}",
            stderr(&output)
        );
    }

    Ok(())
}

#[test]
fn diff_subcommand_with_staged_changes_fails_only_after_ui_startup_without_tty() -> Result<()> {
    let repo = TestRepo::new()?;
    repo.commit_file("src/lib.rs", "fn clean() {}\n", "initial", 1)?;
    repo.write_file("src/lib.rs", "fn staged() {}\n")?;
    repo.stage_file("src/lib.rs")?;

    let output = run_command(gitlogue_command().args([
        "--path",
        repo_path(&repo).to_str().unwrap(),
        "diff",
    ]))?;

    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("No such device or address"));

    Ok(())
}

#[test]
fn diff_subcommand_with_invalid_theme_fails_before_ui_startup() -> Result<()> {
    let repo = TestRepo::new()?;
    repo.commit_file("src/lib.rs", "fn clean() {}\n", "initial", 1)?;
    repo.write_file("src/lib.rs", "fn staged() {}\n")?;
    repo.stage_file("src/lib.rs")?;

    let output = run_command(gitlogue_command().args([
        "--path",
        repo_path(&repo).to_str().unwrap(),
        "diff",
        "--theme",
        "missing-theme",
    ]))?;

    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("Unknown theme: missing-theme"));
    assert!(stderr(&output).contains("Available themes:"));
    assert!(!stderr(&output).contains("No such device or address"));

    Ok(())
}

#[test]
fn default_playback_fails_only_after_ui_startup_without_tty() -> Result<()> {
    let repo = TestRepo::new()?;
    repo.commit_file("src/main.rs", "fn main() {}\n", "initial", 1)?;

    let output =
        run_command(gitlogue_command().args(["--path", repo_path(&repo).to_str().unwrap()]))?;

    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("No such device or address"));

    Ok(())
}

#[test]
fn default_playback_with_invalid_theme_fails_before_ui_startup() -> Result<()> {
    let repo = TestRepo::new()?;
    repo.commit_file("src/main.rs", "fn main() {}\n", "initial", 1)?;

    let output = run_command(gitlogue_command().args([
        "--path",
        repo_path(&repo).to_str().unwrap(),
        "--theme",
        "missing-theme",
    ]))?;

    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("Unknown theme: missing-theme"));
    assert!(stderr(&output).contains("Available themes:"));
    assert!(!stderr(&output).contains("No such device or address"));

    Ok(())
}

#[test]
fn default_playback_quits_cleanly_in_pseudo_tty() -> Result<()> {
    let home = TempHome::new()?;
    let repo = TestRepo::new()?;
    repo.commit_file("src/main.rs", "fn main() {}\n", "initial", 1)?;

    let output = run_in_pseudo_tty(&home, &["--path", repo_path(&repo).to_str().unwrap()], b"q")?;

    assert!(output.status.success());
    assert_eq!(stderr(&output), "");

    Ok(())
}

#[test]
fn diff_subcommand_with_staged_changes_quits_cleanly_in_pseudo_tty() -> Result<()> {
    let home = TempHome::new()?;
    let repo = TestRepo::new()?;
    repo.commit_file("src/lib.rs", "fn clean() {}\n", "initial", 1)?;
    repo.write_file("src/lib.rs", "fn staged() {}\n")?;
    repo.stage_file("src/lib.rs")?;

    let output = run_in_pseudo_tty(
        &home,
        &["--path", repo_path(&repo).to_str().unwrap(), "diff"],
        b"q",
    )?;

    assert!(output.status.success());
    assert_eq!(stderr(&output), "");

    Ok(())
}
