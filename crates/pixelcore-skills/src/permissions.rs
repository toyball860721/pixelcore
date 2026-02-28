//! Permission management system for Skills

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Permission types that Skills can require
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Permission {
    /// File system access permission
    FileSystem {
        /// Base path that can be accessed
        path: PathBuf,
        /// Allow read operations
        read: bool,
        /// Allow write operations
        write: bool,
    },

    /// Network access permission
    Network {
        /// Host or domain (e.g., "api.example.com", "*.example.com", "*" for any)
        host: String,
        /// Port number (0 for any port)
        port: u16,
    },

    /// Compute permission (for resource-intensive operations)
    Compute {
        /// Maximum execution time in milliseconds
        max_time_ms: u64,
        /// Maximum memory usage in MB (0 for unlimited)
        max_memory_mb: u64,
    },

    /// Storage access permission
    Storage {
        /// Storage namespace (e.g., "user_data", "cache")
        namespace: String,
        /// Allow read operations
        read: bool,
        /// Allow write operations
        write: bool,
    },

    /// Process execution permission
    Process {
        /// Allowed command (e.g., "python3", "node")
        command: String,
        /// Allowed arguments pattern (e.g., "*.py" for Python scripts)
        args_pattern: Option<String>,
    },
}

impl Permission {
    /// Check if this permission allows the given operation
    pub fn allows(&self, operation: &PermissionCheck) -> bool {
        match (self, operation) {
            // File system permissions
            (
                Permission::FileSystem { path: allowed_path, read, write },
                PermissionCheck::FileSystem { path: requested_path, operation }
            ) => {
                // Check if requested path is within allowed path
                if !requested_path.starts_with(allowed_path) {
                    return false;
                }

                // Check operation type
                match operation {
                    FileOperation::Read => *read,
                    FileOperation::Write => *write,
                }
            }

            // Network permissions
            (
                Permission::Network { host: allowed_host, port: allowed_port },
                PermissionCheck::Network { host: requested_host, port: requested_port }
            ) => {
                // Check port
                if *allowed_port != 0 && allowed_port != requested_port {
                    return false;
                }

                // Check host (support wildcards)
                if allowed_host == "*" {
                    return true;
                }

                if allowed_host.starts_with("*.") {
                    let domain = &allowed_host[2..];
                    requested_host.ends_with(domain)
                } else {
                    allowed_host == requested_host
                }
            }

            // Compute permissions
            (
                Permission::Compute { max_time_ms, max_memory_mb },
                PermissionCheck::Compute { estimated_time_ms, estimated_memory_mb }
            ) => {
                (*max_time_ms == 0 || estimated_time_ms <= max_time_ms) &&
                (*max_memory_mb == 0 || estimated_memory_mb <= max_memory_mb)
            }

            // Storage permissions
            (
                Permission::Storage { namespace: allowed_ns, read, write },
                PermissionCheck::Storage { namespace: requested_ns, operation }
            ) => {
                if allowed_ns != requested_ns {
                    return false;
                }

                match operation {
                    StorageOperation::Read => *read,
                    StorageOperation::Write => *write,
                }
            }

            // Process permissions
            (
                Permission::Process { command: allowed_cmd, args_pattern },
                PermissionCheck::Process { command: requested_cmd, args }
            ) => {
                if allowed_cmd != requested_cmd {
                    return false;
                }

                // Check args pattern if specified
                if let Some(pattern) = args_pattern {
                    if let Some(args) = args {
                        // Simple wildcard matching
                        if pattern == "*" {
                            return true;
                        }

                        // Check if args match pattern
                        return args.iter().any(|arg| {
                            if pattern.starts_with("*.") {
                                let ext = &pattern[1..];
                                arg.ends_with(ext)
                            } else {
                                arg == pattern
                            }
                        });
                    }
                }

                true
            }

            _ => false,
        }
    }
}

/// Permission check request
#[derive(Debug, Clone)]
pub enum PermissionCheck {
    FileSystem {
        path: PathBuf,
        operation: FileOperation,
    },
    Network {
        host: String,
        port: u16,
    },
    Compute {
        estimated_time_ms: u64,
        estimated_memory_mb: u64,
    },
    Storage {
        namespace: String,
        operation: StorageOperation,
    },
    Process {
        command: String,
        args: Option<Vec<String>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOperation {
    Read,
    Write,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageOperation {
    Read,
    Write,
}

/// Permission manager that checks if operations are allowed
#[derive(Debug, Clone)]
pub struct PermissionManager {
    /// Granted permissions
    permissions: Vec<Permission>,
    /// Whether to allow all operations (unsafe, for testing only)
    allow_all: bool,
}

impl PermissionManager {
    /// Create a new permission manager with no permissions
    pub fn new() -> Self {
        Self {
            permissions: Vec::new(),
            allow_all: false,
        }
    }

    /// Create a permission manager that allows all operations (unsafe)
    pub fn allow_all() -> Self {
        Self {
            permissions: Vec::new(),
            allow_all: true,
        }
    }

    /// Grant a permission
    pub fn grant(&mut self, permission: Permission) {
        self.permissions.push(permission);
    }

    /// Grant multiple permissions
    pub fn grant_all(&mut self, permissions: Vec<Permission>) {
        self.permissions.extend(permissions);
    }

    /// Check if an operation is allowed
    pub fn check(&self, operation: &PermissionCheck) -> bool {
        if self.allow_all {
            return true;
        }

        self.permissions.iter().any(|perm| perm.allows(operation))
    }

    /// Get all granted permissions
    pub fn permissions(&self) -> &[Permission] {
        &self.permissions
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filesystem_permission() {
        let perm = Permission::FileSystem {
            path: PathBuf::from("/tmp"),
            read: true,
            write: false,
        };

        // Should allow read in /tmp
        assert!(perm.allows(&PermissionCheck::FileSystem {
            path: PathBuf::from("/tmp/test.txt"),
            operation: FileOperation::Read,
        }));

        // Should not allow write
        assert!(!perm.allows(&PermissionCheck::FileSystem {
            path: PathBuf::from("/tmp/test.txt"),
            operation: FileOperation::Write,
        }));

        // Should not allow access outside /tmp
        assert!(!perm.allows(&PermissionCheck::FileSystem {
            path: PathBuf::from("/etc/passwd"),
            operation: FileOperation::Read,
        }));
    }

    #[test]
    fn test_network_permission() {
        let perm = Permission::Network {
            host: "*.example.com".to_string(),
            port: 443,
        };

        // Should allow api.example.com:443
        assert!(perm.allows(&PermissionCheck::Network {
            host: "api.example.com".to_string(),
            port: 443,
        }));

        // Should not allow different port
        assert!(!perm.allows(&PermissionCheck::Network {
            host: "api.example.com".to_string(),
            port: 80,
        }));

        // Should not allow different domain
        assert!(!perm.allows(&PermissionCheck::Network {
            host: "api.other.com".to_string(),
            port: 443,
        }));
    }

    #[test]
    fn test_permission_manager() {
        let mut manager = PermissionManager::new();

        // Initially no permissions
        assert!(!manager.check(&PermissionCheck::FileSystem {
            path: PathBuf::from("/tmp/test.txt"),
            operation: FileOperation::Read,
        }));

        // Grant permission
        manager.grant(Permission::FileSystem {
            path: PathBuf::from("/tmp"),
            read: true,
            write: false,
        });

        // Now should allow
        assert!(manager.check(&PermissionCheck::FileSystem {
            path: PathBuf::from("/tmp/test.txt"),
            operation: FileOperation::Read,
        }));
    }

    #[test]
    fn test_allow_all() {
        let manager = PermissionManager::allow_all();

        // Should allow everything
        assert!(manager.check(&PermissionCheck::FileSystem {
            path: PathBuf::from("/etc/passwd"),
            operation: FileOperation::Write,
        }));

        assert!(manager.check(&PermissionCheck::Network {
            host: "evil.com".to_string(),
            port: 666,
        }));
    }
}
