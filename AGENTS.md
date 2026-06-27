# Agent Contribution Rules

## Environment
- This project should support Linux only. Everything will be ran in a container. It does not need to be compiled on macOS.

## Code Style
- Try to limit use of unsafe code.
- Write idiomatic Rust code.
- Avoid writing comments. The code should be self-explanatory. Use named constants instead of magic numbers.
- Keep comments for anything that is not self-explanatory.
- Commits should use the semantic convention (feat:, docs: etc)

## TDD
- Code this project using Test Driven Development (test first, red, green, refactor). Commit after each passing test. Do not create more than one test at a time.
- If you ever run an ad-hoc test consider making it a permanent test if regression is not expected.

## Project Structure
- Ensure you update README.md, Makefile and CI/CD if changes warrant it.
