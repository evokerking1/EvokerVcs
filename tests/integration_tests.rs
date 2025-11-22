use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_repository_init() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize repository
    evokervcs::vcs::operations::init(repo_path).unwrap();

    // Check that .evk directory was created
    assert!(repo_path.join(".evk").exists());
    assert!(repo_path.join(".evk/objects").exists());
    assert!(repo_path.join(".evk/refs").exists());
    assert!(repo_path.join(".evk/HEAD").exists());

    // Check HEAD content
    let head_content = fs::read_to_string(repo_path.join(".evk/HEAD")).unwrap();
    assert_eq!(head_content, "ref: refs/heads/main\n");
}

#[test]
fn test_add_and_status() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize repository
    evokervcs::vcs::operations::init(repo_path).unwrap();

    // Create a test file
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "Hello, EvokerVcs!").unwrap();

    // Add the file
    evokervcs::vcs::operations::add(repo_path, vec![PathBuf::from("test.txt")]).unwrap();

    // Check status
    let status = evokervcs::vcs::operations::status(repo_path).unwrap();
    assert!(status.iter().any(|s| s.contains("test.txt")));
}

#[test]
fn test_commit() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize repository
    evokervcs::vcs::operations::init(repo_path).unwrap();

    // Create and add a test file
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "Hello, EvokerVcs!").unwrap();
    evokervcs::vcs::operations::add(repo_path, vec![PathBuf::from("test.txt")]).unwrap();

    // Commit
    let result = evokervcs::vcs::operations::commit(
        repo_path,
        "Test commit".to_string(),
        "Test User".to_string(),
    );
    assert!(result.is_ok());

    // Check that commit was created
    let repo = evokervcs::vcs::Repository::open(repo_path).unwrap();
    let commit_id = repo.current_commit().unwrap();
    assert!(commit_id.is_some());
}

#[test]
fn test_multiple_commits() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize repository
    evokervcs::vcs::operations::init(repo_path).unwrap();

    // First commit
    let test_file1 = repo_path.join("file1.txt");
    fs::write(&test_file1, "File 1").unwrap();
    evokervcs::vcs::operations::add(repo_path, vec![PathBuf::from("file1.txt")]).unwrap();
    evokervcs::vcs::operations::commit(
        repo_path,
        "First commit".to_string(),
        "Test User".to_string(),
    )
    .unwrap();

    let first_commit = evokervcs::vcs::Repository::open(repo_path)
        .unwrap()
        .current_commit()
        .unwrap()
        .unwrap();

    // Second commit
    let test_file2 = repo_path.join("file2.txt");
    fs::write(&test_file2, "File 2").unwrap();
    evokervcs::vcs::operations::add(repo_path, vec![PathBuf::from("file2.txt")]).unwrap();
    evokervcs::vcs::operations::commit(
        repo_path,
        "Second commit".to_string(),
        "Test User".to_string(),
    )
    .unwrap();

    let second_commit = evokervcs::vcs::Repository::open(repo_path)
        .unwrap()
        .current_commit()
        .unwrap()
        .unwrap();

    // Commits should be different
    assert_ne!(first_commit, second_commit);
}
