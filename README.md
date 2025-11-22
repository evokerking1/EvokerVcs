# EvokerVcs

A custom git-like version control system written in Rust with a Terminal User Interface (TUI) and git compatibility mode.

## Features

- **Custom VCS Implementation**: Full-featured version control system with support for:
  - Repository initialization
  - File staging (add)
  - Commits with history tracking
  - Status checking
  - Object storage (blobs, trees, commits)
  
- **Git Compatibility Mode**: Automatically detects and works with existing git repositories using libgit2
  - Read git repository status
  - View git commit history
  - Check current branch
  
- **Interactive TUI**: Beautiful terminal user interface built with Ratatui
  - Repository status view
  - Commit history viewer
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
```

Stage files for the next commit.

#### Commit changes

```bash
evokervcs commit -m "Your commit message" --author "Your Name"
```

Create a new commit with staged changes.

#### View repository status

```bash
evokervcs status
```

Show the current state of the repository, including staged files and current branch.

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
- `3` - Switch to Staging area (placeholder)
- `h` - Show help screen
- `↑` or `k` - Move selection up
- `↓` or `j` - Move selection down
- `r` - Refresh current view
- `q` - Quit the application

## Architecture

### Core Components

1. **VCS Module** (`src/vcs/`)
   - `objects.rs`: Git-like object model (Blob, Tree, Commit)
   - `repository.rs`: Repository management
   - `index.rs`: Staging area implementation
   - `operations.rs`: Core VCS operations (init, add, commit, status)

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

### Repository Structure

```
.evk/
├── objects/          # Object database
├── refs/
│   └── heads/        # Branch references
├── HEAD              # Current branch pointer
└── index             # Staging area (JSON format)
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

## Known Limitations

- **Directory Support**: Currently, files must be added individually. Directory recursion is not yet implemented.
- **Log for Native Format**: The log view in TUI only works in git compatibility mode. Native EvokerVcs log viewing is planned for a future release.
- **Object Reading**: Object deserialization currently has limited support and always returns Blob objects. Full support for reading Tree and Commit objects is in progress.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details

## Acknowledgments

- Inspired by Git's elegant design
- Built with [Ratatui](https://github.com/ratatui-org/ratatui) for the TUI
- Uses [libgit2](https://libgit2.org/) for git compatibility