# Contributing to Jag IDE

Thank you for your interest in contributing to Jag IDE! We welcome contributions from the community to help make this the best autonomous development platform in the world.

## 🛠️ Development Setup

1. **Clone the repo**:
   ```bash
   git clone https://github.com/JaggaaDaaku/jag-ide.git
   cd jag-ide
   ```

2. **Backend Setup**:
   - Install Rust: https://rustup.rs/
   - Run tests: `cargo test --workspace`
   - Run clippy: `cargo clippy --workspace -- -D warnings`

3. **Frontend Setup**:
   - Install Node.js 18+
   - `cd frontend`
   - `npm install`
   - `npm run dev`

## 🐛 Reporting Bugs

If you find a bug, please use the [Bug Report template](.github/ISSUE_TEMPLATE/bug_report.md) and include:
- Steps to reproduce
- Expected vs. actual behavior
- Your OS and Jag IDE version
- Relevant logs from the terminal or Mission Control

## ✨ Feature Requests

We love hearing new ideas! Please use the [Feature Request template](.github/ISSUE_TEMPLATE/feature_request.md) to describe your proposal.

## 🛡️ Pull Request Guidelines

1. **Create a branch**: Use a descriptive name like `feat/new-agent` or `fix/db-leak`.
2. **Follow the code style**: Run `cargo fmt` and `cargo clippy` before submitting.
3. **Add tests**: Any new feature should include unit or integration tests.
4. **Update documentation**: If you change a user-facing feature, update the README or User Guide.

## 📄 License

By contributing, you agree that your contributions will be licensed under the project's [MIT License](LICENSE).
