# Contributing to Verostark

Thank you for your interest in contributing to Verostark! We want to make it as easy as possible for you to get involved.

## 🛠️ Development Setup

1.  **Install Rust**: We use the latest stable version.
    ```bash
    rustup update
    ```

2.  **Database**:
    We use PostgreSQL. You can start a local instance using Docker:
    ```bash
    ./dev_postgres.sh
    ```

3.  **Download Models**:
    The AI Scrubber requires local model files. Run the download script:
    ```bash
    ./download_model.sh
    ```

4.  **Run Tests**:
    Ensure all tests pass before submitting a PR.
    ```bash
    cargo test
    ```

## 📐 Code Style

*   We use `rustfmt` for code formatting. Please run `cargo fmt` before committing.
*   We use `clippy` for linting. Run `cargo clippy` to catch common issues.

## 🔏 Security Vulnerabilities

If you discover a security vulnerability within Verostark, e.g. a PII scrubbing bypass, please send an email to security@verostark.com instead of creating a public issue.

## 📜 License

By contributing, you agree that your contributions will be licensed under the MIT License.
