# Examples

This directory contains example workflows and usage patterns for EvokerVcs.

## Basic Workflow Example

```bash
# Initialize a new repository
evokervcs init my-project
cd my-project

# Create some files
echo "# My Project" > README.md
echo "print('Hello, World!')" > hello.py

# Add files to staging
evokervcs add README.md hello.py

# Check status
evokervcs status

# Commit changes
evokervcs commit -m "Initial commit" --author "Your Name"

# Launch TUI to view repository
evokervcs tui
```

## Git Compatibility Example

```bash
# Clone any existing git repository
git clone https://github.com/example/repo.git
cd repo

# Use EvokerVcs to view its status
evokervcs status

# Launch TUI to browse git history
evokervcs tui
```

## TUI Navigation Example

After launching `evokervcs tui`:

1. Press `1` to view repository status
2. Press `2` to browse commit history
3. Use `↑/↓` or `k/j` to navigate
4. Press `r` to refresh the current view
5. Press `h` to view help
6. Press `q` to quit

## Advanced Usage

### Working with Multiple Files

```bash
# Add all files in a directory (one by one for now)
for file in $(find . -type f -not -path './.evk/*'); do
    evokervcs add "$file"
done

# Commit everything
evokervcs commit -m "Add all files" --author "Your Name"
```

### Checking Repository Structure

```bash
# View the .evk directory structure
tree .evk/

# Example output:
# .evk/
# ├── HEAD
# ├── index
# ├── objects/
# │   ├── 1e/
# │   │   └── 66c474960a62e88e41f70a2b69e3f60e27b4db
# │   └── 68/
# │       └── cb2e8a87703d957efc7d303fa3cc03f213c11e
# └── refs/
#     └── heads/
#         └── main
```

### Git Compatibility Mode

EvokerVcs automatically detects git repositories:

```bash
# In a git repository
cd /path/to/git/repo

# EvokerVcs TUI will show [GIT COMPAT] in the header
evokervcs tui

# You can view git status and log through EvokerVcs
```
