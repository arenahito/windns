# AGENTS.md

## Project Overview

Windows DNS configuration tool built with Rust and Dioxus.

## Project Structure

```yaml
src:
  components:     # UI components
  dns:            # DNS configuration logic
  app.rs:         # Main application
  main.rs:        # Entry point
  state.rs:       # Application state
assets:
  main.css:       # Styles
```

## Development Rules

### Code Quality

After making code changes, run the following commands:

1. **Format**: `cargo fmt`
2. **Lint**: `cargo clippy`
3. **Test**: `cargo test`

Ensure both commands pass without errors before committing.

### Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>: <description>
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

### Build

Development:
```bash
dx serve
```

Production:
```bash
dx build --release
```

### Bundle

```bash
dx bundle --platform desktop
```

### Test

```bash
cargo test
```

### Code Coverage

```bash

# Get coverage (console output)
cargo llvm-cov

# Show lines with insufficient coverage
cargo llvm-cov --show-missing-lines
```
