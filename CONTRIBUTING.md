# Contributing to Daydream Initiative

Thank you for your interest in contributing to the Daydream Initiative! This document provides guidelines for contributing to this open-source educational technology project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Process](#development-process)
- [Coding Standards](#coding-standards)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)

## Code of Conduct

This project is an educational platform designed to create psychologically safe learning environments. We extend that same principle to our development community:

- **Be respectful** of differing viewpoints and experiences
- **Be collaborative** - help others learn and grow
- **Be patient** - remember that everyone has different levels of expertise
- **Be constructive** - focus on what is best for the project and learners

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:

   ```bash
   git clone https://github.com/YOUR_USERNAME/Day_Dream.git
   cd Day_Dream
   ```

3. **Set up your development environment** - See [SETUP.md](SETUP.md)
4. **Create a feature branch**:

   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Process

### Before You Start

1. **Check existing issues** - Your idea may already be in progress
2. **Open an issue** for discussion if adding a major feature
3. **Comment on an issue** you'd like to work on to avoid duplication

### While Developing

1. **Run tests frequently**:

   ```bash
   cargo test --workspace
   ```

2. **Check code quality**:

   ```bash
   # Format code
   cargo fmt --all

   # Run linter
   cargo clippy --workspace -- -D warnings

   # Check compilation
   cargo check --workspace
   ```

3. **Test locally**:
   - Backend: `cd backend && cargo run`
   - Frontend: `cd frontend && trunk serve`

## Coding Standards

### Rust Style Guide

We follow the official [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/):

- **Use `rustfmt`** - Code must pass `cargo fmt --check`
- **Use `clippy`** - Code must pass `cargo clippy` with no warnings
- **Maximum line length**: 100 characters
- **Naming conventions**:
  - `snake_case` for function names, variables
  - `PascalCase` for types, traits
  - `SCREAMING_SNAKE_CASE` for constants

### Architecture Principles

#### **CRITICAL**: Async/Sync Bridging

When working with Axum handlers that need to access Bevy ECS state:

```rust
// ❌ INCORRECT - Will deadlock
async fn handler(State(world): State<World>) {
    // Cannot directly access sync World from async
}

// ✅ CORRECT - Use bevy_defer bridge
async fn handler(State(async_world): State<AsyncWorld>) {
    async_world.apply(|world: &mut World| {
        // Safe mutation
    }).await;
}
```

#### **CRITICAL**: Blocking Operations

Never block the Tokio runtime:

```rust
// ❌ INCORRECT - Blocks entire server
async fn handler() {
    let result = expensive_ai_task(); // Blocks!
}

// ✅ CORRECT - Use spawn_blocking
async fn handler() {
    let result = tokio::task::spawn_blocking(move || {
        expensive_ai_task()
    }).await?;
}
```

### Frontend (Leptos) Guidelines

- **Use Leptos 0.8 APIs**:
  - `signal()` not `create_signal()`
  - `Callback::run()` not `.call()`
  - `path!()` macro for routes
  - Import from `leptos::prelude::*`

- **Component structure**:

  ```rust
  #[component]
  pub fn MyComponent() -> impl IntoView {
      view! { /* JSX-like syntax */ }
  }
  ```

- **Reactive signals**:

  ```rust
  let (count, set_count) = signal(0);
  // Use .get() to read, .set() or .update() to write
  ```

### Documentation

- **Public APIs** must have doc comments:

  ```rust
  /// Saves a story graph to the database.
  ///
  /// # Arguments
  /// * `story_id` - Unique identifier for the story
  /// * `graph_data` - JSON-serialized graph structure
  ///
  /// # Errors
  /// Returns `DatabaseError` if save fails
  pub async fn save_graph(...)
  ```

- **Complex logic** should have inline comments explaining *why*, not *what*

## Commit Guidelines

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style changes (formatting, no logic change)
- `refactor`: Code restructuring (no feature change)
- `perf`: Performance improvement
- `test`: Adding/updating tests
- `chore`: Build process, dependencies, etc.

### Examples

```
feat(frontend): add story graph save dialog

- Implement modal component
- Add save button to canvas
- Wire up API call

Closes #123
```

```
fix(backend): prevent race condition in reflection sharing

Use tokio::sync::Mutex instead of std::sync::Mutex to avoid
blocking the async runtime when learners share reflections.

Fixes #456
```

## Pull Request Process

### Before Submitting

1. **Rebase** on latest `main`:

   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run full test suite**:

   ```bash
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cargo fmt --all --check
   ```

3. **Update documentation** if needed:
   - README.md
   - SETUP.md
   - Inline code docs

### PR Description Template

```markdown
## Description
<!-- What does this PR do? -->

## Motivation
<!-- Why is this change needed? -->

## Related Issues
<!-- Link to issues: Closes #XX, Fixes #YY -->

## Testing
<!-- How was this tested? -->
- [ ] Unit tests added
- [ ] Integration tests added
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-reviewed my own code
- [ ] Commented complex code
- [ ] Updated documentation
- [ ] No new warnings
- [ ] Tests pass locally
```

### Review Process

1. **At least one maintainer** must approve
2. **All CI checks** must pass
3. **No merge conflicts** with `main`
4. Maintainer will merge using **squash and merge**

## Areas Where We Need Help

### High Priority

- **Local AI Integration**: Implementing Whisper STT / OpenAudio TTS with `rocm-rs`
- **Vector Database**: LanceDB integration for semantic search
- **VaaM System**: Vocabulary-as-a-Mechanic implementation
- **Mentor Portal**: Secure,  FERPA-compliant messaging system

### Medium Priority

- **UI/UX Polish**: Improving the authoring interface
- **Documentation**: Tutorials, examples, API docs
- **Testing**: Increasing test coverage
- **Accessibility**: WCAG 2.1 AA compliance

### Good First Issues

Look for issues labeled `good first issue` - these are specifically chosen for newcomers!

## Questions?

- **Technical questions**: Open a GitHub issue with the `question` label
- **Security concerns**: Email (to be provided) - do NOT open public issues
- **General discussion**: GitHub Discussions tab

## License

By contributing, you agree that your contributions will be licensed under the GNU General Public License v3.0 (GPLv3). This ensures the project remains open-source and prevents proprietary forks.

---

**Thank you for helping build ethical, privacy-first educational technology! 🎓**
