# spar-command

A dependency-free data model for command and pipeline execution plans, shared between the `spar` compiler, which builds plans from Spar source, and `spar-process`, which runs them.

## What's here

- `plan` module: command, pipeline, and redirection plan types. No Spar runtime, no process-execution code, no I/O.

## Why a separate crate

Keeping the plan types free of both the compiler and the executor lets each depend on the same shape without pulling in the other.

Part of the [Spar](https://github.com/oraclevs/spar) toolchain.
License: MIT
