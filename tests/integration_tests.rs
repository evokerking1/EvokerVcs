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

#[test]
fn test_directory_recursion() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize repository
    evokervcs::vcs::operations::init(repo_path).unwrap();

    // Create directory structure
    let src_dir = repo_path.join("src");
    let utils_dir = src_dir.join("utils");
    fs::create_dir_all(&utils_dir).unwrap();

    // Create files
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();
    fs::write(utils_dir.join("helper.rs"), "// helper").unwrap();
    fs::write(utils_dir.join("common.rs"), "// common").unwrap();

    // Add the entire src directory
    evokervcs::vcs::operations::add(repo_path, vec![PathBuf::from("src")]).unwrap();

    // Check that all files were added
    let status = evokervcs::vcs::operations::status(repo_path).unwrap();
    let status_text = status.join("\n");
    
    assert!(status_text.contains("src/main.rs"));
    assert!(status_text.contains("src/utils/helper.rs"));
    assert!(status_text.contains("src/utils/common.rs"));
}

#[test]
fn test_log_functionality() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize repository
    evokervcs::vcs::operations::init(repo_path).unwrap();

    // Create first commit
    let file1 = repo_path.join("file1.txt");
    fs::write(&file1, "File 1").unwrap();
    evokervcs::vcs::operations::add(repo_path, vec![PathBuf::from("file1.txt")]).unwrap();
    evokervcs::vcs::operations::commit(
        repo_path,
        "First commit".to_string(),
        "Author 1".to_string(),
    )
    .unwrap();

    // Create second commit
    let file2 = repo_path.join("file2.txt");
    fs::write(&file2, "File 2").unwrap();
    evokervcs::vcs::operations::add(repo_path, vec![PathBuf::from("file2.txt")]).unwrap();
    evokervcs::vcs::operations::commit(
        repo_path,
        "Second commit".to_string(),
        "Author 2".to_string(),
    )
    .unwrap();

    // Get log
    let log = evokervcs::vcs::operations::log(repo_path, 10).unwrap();

    // Should have 2 commits
    assert_eq!(log.len(), 2);

    // Most recent commit should be first
    assert_eq!(log[0].message, "Second commit");
    assert_eq!(log[0].author, "Author 2");

    // Second commit should be next
    assert_eq!(log[1].message, "First commit");
    assert_eq!(log[1].author, "Author 1");
}

#[test]
fn test_object_reading() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize repository
    evokervcs::vcs::operations::init(repo_path).unwrap();

    // Create and commit a file
    let file = repo_path.join("test.txt");
    fs::write(&file, "test content").unwrap();
    evokervcs::vcs::operations::add(repo_path, vec![PathBuf::from("test.txt")]).unwrap();
    evokervcs::vcs::operations::commit(
        repo_path,
        "Test commit".to_string(),
        "Test Author".to_string(),
    )
    .unwrap();

    // Get commit ID
    let repo = evokervcs::vcs::Repository::open(repo_path).unwrap();
    let commit_id = repo.current_commit().unwrap().unwrap();
    let commit_oid = evokervcs::vcs::objects::ObjectId::new(commit_id);

    // Read commit object
    let obj = evokervcs::vcs::objects::Object::read_from_file(&repo.objects_dir, &commit_oid);
    assert!(obj.is_ok());
    
    // Verify it's a commit object
    match obj.unwrap() {
        evokervcs::vcs::objects::Object::Commit(commit) => {
            assert_eq!(commit.message, "Test commit");
            assert_eq!(commit.author, "Test Author");
        }
        _ => panic!("Expected commit object"),
    }
}
