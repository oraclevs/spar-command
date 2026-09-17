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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPlan {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<EnvironmentOverride>,
    pub cwd: Option<WorkingDirectory>,
    pub stdin: Option<Redirection>,
    pub stdout: Option<Redirection>,
    pub stderr: Option<Redirection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelinePlan {
    pub commands: Vec<CommandPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Command(CommandPlan),
    Pipeline(PipelinePlan),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Join {
    Always,
    OnSuccess,
    OnFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellPlan {
    pub steps: Vec<(Join, Step)>,
}

impl ShellPlan {
    /// Immutable sequential composition — the runtime behavior behind
    /// Spar's `shell + shell -> shell` operator (master prompt §15-16).
    /// `other`'s first step is rewritten to `Join::Always`, matching ordinary
    /// `;` sequencing. Conditional execution remains explicit through `&&`
    /// and `||` joins inside either operand.
    pub fn then(mut self, other: ShellPlan) -> ShellPlan {
        let mut other_steps = other.steps.into_iter();
        if let Some((_, first_step)) = other_steps.next() {
            self.steps.push((Join::Always, first_step));
        }
        self.steps.extend(other_steps);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn then_appends_steps_in_order() {
        let a = ShellPlan {
            steps: vec![(Join::Always, Step::Command(cmd("cargo", &["fmt"])))],
        };
        let b = ShellPlan {
            steps: vec![(Join::Always, Step::Command(cmd("cargo", &["build"])))],
        };
        let combined = a.then(b);
        assert_eq!(combined.steps.len(), 2);
        assert_eq!(combined.steps[0].1, Step::Command(cmd("cargo", &["fmt"])));
        assert_eq!(combined.steps[1].1, Step::Command(cmd("cargo", &["build"])));
    }

    #[test]
    fn then_runs_the_first_step_of_the_second_plan_unconditionally() {
        let a = ShellPlan {
            steps: vec![(Join::Always, Step::Command(cmd("a", &[])))],
        };
        let b = ShellPlan {
            steps: vec![
                (Join::Always, Step::Command(cmd("b1", &[]))),
                (Join::OnFailure, Step::Command(cmd("b2", &[]))),
            ],
        };
        let combined = a.then(b);
        assert_eq!(
            combined.steps[0].0,
            Join::Always,
            "first plan's own join untouched"
        );
        assert_eq!(
            combined.steps[1].0,
            Join::Always,
            "sequential composition runs after either prior outcome"
        );
        assert_eq!(
            combined.steps[2].0,
            Join::OnFailure,
            "second plan's later joins untouched"
        );
    }

    #[test]
    fn then_with_empty_left_operand_preserves_an_always_head() {
        let a = ShellPlan { steps: vec![] };
        let b = ShellPlan {
            steps: vec![(Join::Always, Step::Command(cmd("only", &[])))],
        };
        let combined = a.then(b);
        assert_eq!(combined.steps.len(), 1);
        assert_eq!(combined.steps[0].0, Join::Always);
    }

    #[test]
    fn then_with_empty_right_operand_is_a_noop() {
        let a = ShellPlan {
            steps: vec![(Join::Always, Step::Command(cmd("only", &[])))],
        };
        let b = ShellPlan { steps: vec![] };
        let combined = a.clone().then(b);
        assert_eq!(combined, a);
    }

    #[test]
    fn then_does_not_mutate_operands_because_it_takes_them_by_value() {
        let a = ShellPlan {
            steps: vec![(Join::Always, Step::Command(cmd("a", &[])))],
        };
        let b = ShellPlan {
            steps: vec![(Join::Always, Step::Command(cmd("b", &[])))],
        };
        let combined_from_clones = a.clone().then(b.clone());
        let combined_direct = a.then(b);
        assert_eq!(combined_from_clones, combined_direct);
    }

    #[test]
    fn shell_plan_holds_ordered_joined_steps() {
        let plan = ShellPlan {
            steps: vec![
                (
                    Join::Always,
                    Step::Command(cmd("cargo", &["fmt", "--check"])),
                ),
                (Join::OnSuccess, Step::Command(cmd("cargo", &["test"]))),
            ],
        };
        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.steps[0].0, Join::Always);
        assert_eq!(plan.steps[1].0, Join::OnSuccess);
    }

    #[test]
    fn empty_shell_plan_is_constructible() {
        let plan = ShellPlan { steps: vec![] };
        assert!(plan.steps.is_empty());
    }

    #[test]
    fn join_variants_are_distinct() {
        assert_ne!(Join::Always, Join::OnSuccess);
        assert_ne!(Join::OnSuccess, Join::OnFailure);
        assert_ne!(Join::Always, Join::OnFailure);
    }

    fn cmd(program: &str, args: &[&str]) -> CommandPlan {
        CommandPlan {
            program: program.into(),
            args: args.iter().map(|s| s.to_string()).collect(),
            env: vec![],
            cwd: None,
            stdin: None,
            stdout: None,
            stderr: None,
        }
    }

    #[test]
    fn pipeline_plan_holds_ordered_commands() {
        let pipeline = PipelinePlan {
            commands: vec![
                cmd("cat", &["file.log"]),
                cmd("grep", &["ERROR"]),
                cmd("sort", &[]),
            ],
        };
        assert_eq!(pipeline.commands.len(), 3);
        assert_eq!(pipeline.commands[0].program, "cat");
        assert_eq!(pipeline.commands[2].program, "sort");
    }

    #[test]
    fn step_wraps_command_or_pipeline() {
        let step_cmd = Step::Command(cmd("ls", &[]));
        let step_pipe = Step::Pipeline(PipelinePlan {
            commands: vec![cmd("a", &[]), cmd("b", &[])],
        });
        assert!(matches!(step_cmd, Step::Command(_)));
        assert!(matches!(step_pipe, Step::Pipeline(_)));
    }

    #[test]
    fn pipeline_plan_equality() {
        let a = PipelinePlan {
            commands: vec![cmd("a", &[]), cmd("b", &[])],
        };
        let b = PipelinePlan {
            commands: vec![cmd("a", &[]), cmd("b", &[])],
        };
        assert_eq!(a, b);
    }

    #[test]
    fn command_plan_construction_and_field_access() {
        let cmd = CommandPlan {
            program: "cargo".into(),
            args: vec!["build".into(), "--release".into()],
            env: vec![EnvironmentOverride {
                key: "RUST_LOG".into(),
                value: "debug".into(),
            }],
            cwd: Some(WorkingDirectory::Path("/repo".into())),
            stdin: None,
            stdout: Some(Redirection::File {
                path: "build.log".into(),
                mode: RedirectMode::Truncate,
            }),
            stderr: None,
        };
        assert_eq!(cmd.program, "cargo");
        assert_eq!(cmd.args, vec!["build", "--release"]);
        assert_eq!(cmd.env.len(), 1);
    }

    #[test]
    fn command_plan_equality_and_clone() {
        let cmd = CommandPlan {
            program: "echo".into(),
            args: vec!["hi".into()],
            env: vec![],
            cwd: None,
            stdin: None,
            stdout: None,
            stderr: None,
        };
        assert_eq!(cmd.clone(), cmd);
    }

    #[test]
    fn command_plan_no_args_no_overrides() {
        let cmd = CommandPlan {
            program: "pwd".into(),
            args: vec![],
            env: vec![],
            cwd: None,
            stdin: None,
            stdout: None,
            stderr: None,
        };
        assert!(cmd.args.is_empty());
        assert!(cmd.env.is_empty());
        assert!(cmd.cwd.is_none());
    }

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
