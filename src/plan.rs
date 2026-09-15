//! Command/execution-plan data types. See the crate-level doc comment in
//! `lib.rs` for the design doc pointer.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedirectMode {
    Truncate,
    Append,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Redirection {
    File { path: String, mode: RedirectMode },
    DuplicateFd(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentOverride {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkingDirectory {
    Path(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redirection_file_truncate_equality() {
        let a = Redirection::File {
            path: "out.log".into(),
            mode: RedirectMode::Truncate,
        };
        let b = Redirection::File {
            path: "out.log".into(),
            mode: RedirectMode::Truncate,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn redirection_file_append_differs_from_truncate() {
        let a = Redirection::File {
            path: "out.log".into(),
            mode: RedirectMode::Truncate,
        };
        let b = Redirection::File {
            path: "out.log".into(),
            mode: RedirectMode::Append,
        };
        assert_ne!(a, b);
    }

    #[test]
    fn redirection_duplicate_fd_equality() {
        assert_eq!(Redirection::DuplicateFd(1), Redirection::DuplicateFd(1));
        assert_ne!(Redirection::DuplicateFd(1), Redirection::DuplicateFd(2));
    }

    #[test]
    fn environment_override_equality() {
        let a = EnvironmentOverride {
            key: "RUST_LOG".into(),
            value: "debug".into(),
        };
        let b = EnvironmentOverride {
            key: "RUST_LOG".into(),
            value: "debug".into(),
        };
        assert_eq!(a, b);
        let c = EnvironmentOverride {
            key: "RUST_LOG".into(),
            value: "info".into(),
        };
        assert_ne!(a, c);
    }

    #[test]
    fn working_directory_equality() {
        assert_eq!(
            WorkingDirectory::Path("/tmp".into()),
            WorkingDirectory::Path("/tmp".into())
        );
        assert_ne!(
            WorkingDirectory::Path("/tmp".into()),
            WorkingDirectory::Path("/var".into())
        );
    }

    #[test]
    fn primitive_types_are_cloneable() {
        let r = Redirection::DuplicateFd(2);
        let cloned = r.clone();
        assert_eq!(r, cloned);
    }
}
