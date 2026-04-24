# Contributing to Hope OS

We welcome contributions! This guide will help you get started.

## Code of Conduct

- Be respectful and inclusive
- Focus on what's best for the community
- Keep a positive, constructive attitude

## Ways to Contribute

### 🐛 Bug Reports
Use GitHub Issues to report bugs. Include:
- Steps to reproduce
- Expected vs actual behavior
- System information (`hope-ctl report`)

### 💡 Feature Requests
Open an issue with:
- Clear description of the feature
- Use case / why it's needed
- Any relevant mockups or examples

### 📝 Documentation
Help improve docs in `docs/`:
- Fix typos
- Clarify confusing sections
- Add missing examples

### 💻 Code
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## Development Setup

```bash
# Install dependencies
sudo apt install make gcc rustc cargo

# Build HAL+
cd hal
cargo build --release

# Run tests
cargo test
```

## Coding Standards

- Rust: Follow `rustfmt` and `clippy`
- Shell: Use `shellcheck` for scripts
- Keep code clear and well-documented
- No jargon in user-facing strings

## Commit Messages

Use clear, descriptive messages:
```
hal: add vendor:product ID lookup flow

Implements the core detection flow from SPEC.md section 2.
- Monitors udev for device events
- Looks up IDs in SQLite database
- Falls back to cloud sync for unknown devices
```

## Pull Request Process

1. Update documentation if needed
2. Add tests for new functionality
3. Ensure `cargo test` passes
4. Request review from maintainers
5. Address feedback promptly

## Questions?

- GitHub Discussions
- Matrix: #hope-os:matrix.org
- Email: dev@hope-os.org

---

Thank you for contributing to Hope OS!
