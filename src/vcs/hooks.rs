use anyhow::Result;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Hook types supported by EvokerVcs
#[derive(Debug, Clone, Copy)]
pub enum HookType {
    PreCommit,
    PostCommit,
    PrePush,
    PostCheckout,
    PreRebase,
    PostRewrite,
}

impl HookType {
    pub fn name(&self) -> &str {
        match self {
            HookType::PreCommit => "pre-commit",
            HookType::PostCommit => "post-commit",
            HookType::PrePush => "pre-push",
            HookType::PostCheckout => "post-checkout",
            HookType::PreRebase => "pre-rebase",
            HookType::PostRewrite => "post-rewrite",
        }
    }
}

pub struct HookManager {
    hooks_dir: PathBuf,
}

impl HookManager {
    pub fn new(hooks_dir: PathBuf) -> Self {
        HookManager { hooks_dir }
    }

    /// Check if a hook exists and is executable
    pub fn hook_exists(&self, hook_type: HookType) -> bool {
        let hook_path = self.hooks_dir.join(hook_type.name());
        if !hook_path.exists() {
            return false;
        }

        // Check if executable on Unix systems
        #[cfg(unix)]
        {
            if let Ok(metadata) = fs::metadata(&hook_path) {
                let permissions = metadata.permissions();
                return permissions.mode() & 0o111 != 0;
            }
        }

        #[cfg(not(unix))]
        {
            return true;
        }

        false
    }

    /// Run a hook if it exists
    pub fn run_hook(&self, hook_type: HookType, args: &[&str]) -> Result<bool> {
        if !self.hook_exists(hook_type) {
            return Ok(true); // Hook doesn't exist, consider it successful
        }

        let hook_path = self.hooks_dir.join(hook_type.name());
        
        let output = Command::new(&hook_path)
            .args(args)
            .output()?;

        if output.status.success() {
            // Print hook output
            if !output.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            }
            Ok(true)
        } else {
            // Print hook error output
            if !output.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
            }
            Ok(false)
        }
    }

    /// Install a sample hook
    pub fn install_sample_hook(&self, hook_type: HookType, content: &str) -> Result<()> {
        let hook_path = self.hooks_dir.join(format!("{}.sample", hook_type.name()));
        fs::write(&hook_path, content)?;
        
        // Make it executable on Unix
        #[cfg(unix)]
        {
            let mut permissions = fs::metadata(&hook_path)?.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&hook_path, permissions)?;
        }

        Ok(())
    }

    /// Install default sample hooks
    pub fn install_default_samples(&self) -> Result<()> {
        // Pre-commit hook sample
        let pre_commit = r#"#!/bin/sh
# Pre-commit hook sample
# This hook is invoked by evokervcs commit
# Rename to 'pre-commit' (remove .sample) and make executable to enable

# Example: Check for whitespace errors
# Uncomment to enable:
# exec evokervcs diff --check

echo "Running pre-commit hook..."
exit 0
"#;

        // Post-commit hook sample
        let post_commit = r#"#!/bin/sh
# Post-commit hook sample  
# This hook is invoked after a commit
# Rename to 'post-commit' (remove .sample) and make executable to enable

echo "Commit completed successfully!"
exit 0
"#;

        self.install_sample_hook(HookType::PreCommit, pre_commit)?;
        self.install_sample_hook(HookType::PostCommit, post_commit)?;

        Ok(())
    }
}
