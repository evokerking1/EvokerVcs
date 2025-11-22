# EvokerVcs

A custom git-like version control system written in Rust with a Terminal User Interface (TUI) and git compatibility mode.

## Features

- **Custom VCS Implementation**: Full-featured version control system with support for:
  - Repository initialization
  - File staging with directory recursion
  - Commits with history tracking and hooks
  - Branch management (create, delete, rename, checkout)
  - Tag support (lightweight and annotated)
  - Status checking
  - Commit log viewing
  - Object storage (blobs, trees, commits)
  - Git hooks (pre-commit, post-commit, etc.)
  
- **Git Compatibility Mode**: Automatically detects and works with existing git repositories using libgit2
  - Read git repository status
  - View git commit history
  - Check current branch
  
- **Interactive TUI**: Beautiful terminal user interface built with Ratatui
  - Repository status view
  - Commit history viewer
  - Branches view with current branch indicator
  - Tags list view
  - Keyboard navigation (Vim-style + arrow keys)
  - Help screen with all shortcuts

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo package manager

### Build from Source

```bash
git clone https://github.com/evokerking1/EvokerVcs.git
cd EvokerVcs
cargo build --release
```

The binary will be available at `target/release/evokervcs`.

## Usage

### Command Line Interface

#### Initialize a new repository

```bash
evokervcs init [path]
```

Creates a new EvokerVcs repository in the specified directory (defaults to current directory).

#### Add files to staging area

```bash
evokervcs add <file1> <file2> ...
evokervcs add src/          # Add entire directory recursively
evokervcs add docs/ tests/  # Add multiple directories
```

Stage files or entire directories for the next commit. Directories are added recursively.

#### Commit changes

```bash
evokervcs commit -m "Your commit message" --author "Your Name"
```

Create a new commit with staged changes. Runs pre-commit and post-commit hooks if configured.

#### View repository status

```bash
evokervcs status
```

Show the current state of the repository, including staged files and current branch.

#### View commit history

```bash
evokervcs log [--limit 10]
```

Display the commit history with author, date, and commit message. Use `--limit` to control how many commits to show.

#### Branch management

```bash
evokervcs branch list                    # List all branches
evokervcs branch create <name>           # Create new branch
evokervcs branch delete <name> [--force] # Delete branch
evokervcs branch rename <old> <new>      # Rename branch
evokervcs checkout <branch>              # Switch to branch
evokervcs checkout -b <branch>           # Create and switch to new branch
```

#### Tag management

```bash
evokervcs tag list                          # List all tags
evokervcs tag create <name>                 # Create lightweight tag
evokervcs tag create <name> -m "message"    # Create annotated tag
evokervcs tag delete <name>                 # Delete tag
```

### Interactive TUI

Launch the TUI by running:

```bash
evokervcs tui [path]
```

Or simply:

```bash
evokervcs
```

#### TUI Keyboard Shortcuts

- `1` - Switch to Status view
- `2` - Switch to Commit log view
- `3` - Switch to Branches view
- `4` - Switch to Tags view
- `5` - Switch to Staging area
- `h` - Show help screen
- `↑` or `k` - Move selection up
- `↓` or `j` - Move selection down
- `r` - Refresh current view
- `q` - Quit the application

## Architecture

### Core Components

1. **VCS Module** (`src/vcs/`)
   - `objects.rs`: Git-like object model (Blob, Tree, Commit) with full serialization/deserialization
   - `repository.rs`: Repository management with branch and tag operations
   - `index.rs`: Staging area implementation
   - `operations.rs`: Core VCS operations (init, add, commit, status, log)
   - `hooks.rs`: Git hooks system (pre-commit, post-commit, etc.)

2. **Git Compatibility Module** (`src/git_compat/`)
   - Provides integration with existing git repositories
   - Uses libgit2-rs for git operations

3. **TUI Module** (`src/tui/`)
   - `app.rs`: Application state and logic
   - `ui.rs`: UI rendering with Ratatui

### Object Storage

EvokerVcs uses a content-addressable storage system similar to git:
- Objects are stored in `.evk/objects/`
- Each object is identified by its SHA-1 hash
- Objects are compressed using zlib
- Supports three object types: Blob, Tree, and Commit
- Full object deserialization support for reading all object types

### Repository Structure

```
.evk/
├── objects/          # Object database
├── refs/
│   ├── heads/        # Branch references
│   └── tags/         # Tag references
├── hooks/            # Git hooks (executable scripts)
│   ├── pre-commit.sample
│   └── post-commit.sample
├── HEAD              # Current branch pointer
└── index             # Staging area (JSON format)
```

### Git Hooks

EvokerVcs supports git-style hooks for automation:

- **pre-commit**: Runs before creating a commit. Can abort the commit if it exits with non-zero status.
- **post-commit**: Runs after a successful commit.
- **pre-push**: Runs before pushing changes (when implemented).
- **post-checkout**: Runs after checking out a branch.

To enable a hook, rename it from `.sample` to remove the extension and make it executable:

```bash
cd .evk/hooks
mv pre-commit.sample pre-commit
chmod +x pre-commit
```

## Git Compatibility

EvokerVcs automatically detects git repositories and switches to compatibility mode, allowing you to:
- View status of git repositories
- Browse git commit history
- Check current git branch

This makes EvokerVcs a universal tool that can work with both its native format and existing git repositories.

## Example Workflow

```bash
# Create a new project
mkdir my-project
cd my-project

# Initialize repository
evokervcs init

# Create some files
echo "Hello, World!" > hello.txt
echo "# My Project" > README.md

# Stage files
evokervcs add hello.txt README.md

# Check status
evokervcs status

# Commit changes
evokervcs commit -m "Initial commit" --author "John Doe"

# Launch TUI to browse history
evokervcs tui
```

## Development

### Running Tests

```bash
cargo test
```

### Code Structure

- `src/main.rs`: CLI entry point and TUI event loop
- `src/vcs/`: Core version control functionality
- `src/git_compat/`: Git compatibility layer
- `src/tui/`: Terminal UI components

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details

## Acknowledgments

- Inspired by Git's elegant design
- Built with [Ratatui](https://github.com/ratatui-org/ratatui) for the TUI
- Uses [libgit2](https://libgit2.org/) for git compatibility