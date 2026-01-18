# Contributing to Alchemist

First off, thank you for considering contributing to Alchemist! 🧪

## How to Contribute

### Reporting Bugs

If you find a bug, please open an issue on GitHub with:
- A clear, descriptive title
- Steps to reproduce the problem
- Expected vs actual behavior
- Your environment (OS, Rust version)

### Suggesting Features

Feature requests are welcome! Please open an issue with:
- A clear description of the feature
- Use cases and benefits
- Any implementation ideas you might have

### Pull Requests

1. **Fork the repository**
2. **Create a feature branch**
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. **Make your changes**
   - Follow the existing code style
   - Add tests if applicable
   - Update documentation if needed
4. **Run the tests**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```
5. **Commit your changes**
   ```bash
   git commit -m 'Add amazing feature'
   ```
6. **Push and open a Pull Request**
   ```bash
   git push origin feature/amazing-feature
   ```

## Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/Alchemist
cd Alchemist

# Build the project
cargo build

# Run tests
cargo test

# Run with an example
cargo run -- -i examples/simple_config.json -t typescript
```

## Code Style

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Add doc comments for public APIs
- Keep functions small and focused

## Adding a New Generator

To add support for a new output language:

1. Create a new file in `src/generators/` (e.g., `python.rs`)
2. Implement the `CodeGenerator` trait
3. Add the new format to `src/formats.rs`
4. Update the CLI in `src/cli.rs`
5. Add tests and examples

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for helping make Alchemist better! 🦀✨
