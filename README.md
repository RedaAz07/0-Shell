# 0-Shell

A small, educational Unix-like shell written in Rust. This project implements a minimal interactive shell with a handful of common commands and a simple command parser/executor.

**Features**
- **Built-in commands:** `cat`, `cd`, `cp`, `echo`, `exit`, `ls`, `mv`, `pwd_state`, `rm` (see `src/commands/`)
- **Simple parser and executor:** lightweight parsing and command dispatch (see `src/helpers/`)
- **Single-binary distribution:** built with `cargo build` and run as `shell`.

**Project Structure**
- `src/main.rs`: program entrypoint and REPL loop
- `src/commands/`: implementations of individual built-in commands
- `src/helpers/`: parser, executor and small utilities

**Build**
Install Rust (if needed) and build with Cargo:


The release binary will be at `target/release/shell` (or `target/debug/shell` for a debug build).

**Run (development)**

```bash
cargo run --release --  # builds and runs the shell
# or
cargo run
```

Start the shell and try commands like:

```text
> echo Hello world
> ls
> cat README.md
> cd ..
> pwd
> exit
```

**Notes & Development**
- The command implementations live in `src/commands/` and are designed to be small and easy to extend.
- The parser and executor are in `src/helpers/` and can be improved to support pipelines, redirection, and background execution.

If you'd like, I can add examples, tests, or CI configuration next.