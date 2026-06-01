use crate::detect::ProjectType;

pub fn get_claude_md(t: ProjectType) -> &'static str {
    match t {
        ProjectType::TypescriptBun => TYPESCRIPT_BUN_TEMPLATE,
        ProjectType::TypescriptNode => TYPESCRIPT_NODE_TEMPLATE,
        ProjectType::Rust => RUST_TEMPLATE,
        ProjectType::Go => GO_TEMPLATE,
        ProjectType::Python => PYTHON_TEMPLATE,
        ProjectType::Generic => GENERIC_TEMPLATE,
    }
}

pub fn get_plan_command() -> &'static str {
    PLAN_COMMAND
}

pub fn get_review_command() -> &'static str {
    REVIEW_COMMAND
}

const TYPESCRIPT_BUN_TEMPLATE: &str = "# Project Context

## Stack
- Runtime: Bun
- Language: TypeScript

## Commands
- Run: `bun run <file>`
- Test: `bun test`
- Build: `bun build <entry>`
- Install: `bun install`
- Lint/Format: `bunx biome check --write .`

## Conventions
- Use `bun:test` for testing (describe/test/expect)
- Use `Bun.file()` / `Bun.write()` for file I/O
- Use `Bun.serve()` for HTTP servers
- Bun loads .env automatically — no dotenv needed
";

const TYPESCRIPT_NODE_TEMPLATE: &str = "# Project Context

## Stack
- Runtime: Node.js
- Language: TypeScript

## Commands
- Run: `npx tsx <file>`
- Test: `npm test`
- Build: `npm run build`
- Install: `npm install`
- Lint/Format: `npx eslint . && npx prettier --write .`

## Conventions
- Follow existing test framework conventions in the project
- Use async/await over raw promises
- Prefer named exports
";

const RUST_TEMPLATE: &str = "# Project Context

## Stack
- Language: Rust

## Commands
- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt`
- Run: `cargo run`

## Conventions
- Run `cargo clippy` before committing
- Run `cargo fmt` to format code
- Handle errors with Result — avoid unwrap in library code
- Write doc comments for public items
";

const GO_TEMPLATE: &str = "# Project Context

## Stack
- Language: Go

## Commands
- Build: `go build ./...`
- Test: `go test ./...`
- Vet: `go vet ./...`
- Format: `gofmt -w .`

## Conventions
- Run `go vet` before committing
- Use `gofmt` for formatting
- Follow standard Go project layout
- Handle all errors — never ignore returned errors
";

const PYTHON_TEMPLATE: &str = "# Project Context

## Stack
- Language: Python

## Commands
- Test: `pytest`
- Lint: `ruff check .`
- Format: `ruff format .`
- Type check: `mypy .`

## Conventions
- Use type hints for function signatures
- Use pytest for testing
- Use ruff or black for formatting
- Follow PEP 8 style guidelines
";

const GENERIC_TEMPLATE: &str = "# Project Context

## Stack
- TODO: Describe the project stack here

## Commands
- TODO: Add common commands (build, test, lint, run)

## Conventions
- TODO: Add project-specific conventions and guidelines
";

const PLAN_COMMAND: &str = "Review the codebase and create a detailed implementation plan for:

$ARGUMENTS

Include:
1. Files to create or modify
2. Key design decisions
3. Testing approach
4. Potential edge cases
";

const REVIEW_COMMAND: &str =
    "Review the following code changes for bugs, style issues, and improvements:

$ARGUMENTS

Check for:
1. Correctness and edge cases
2. Error handling
3. Naming and readability
4. Performance concerns
5. Security issues
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typescript_bun_mentions_bun() {
        assert!(get_claude_md(ProjectType::TypescriptBun)
            .to_lowercase()
            .contains("bun"));
    }

    #[test]
    fn typescript_node_mentions_node() {
        assert!(get_claude_md(ProjectType::TypescriptNode)
            .to_lowercase()
            .contains("node"));
    }

    #[test]
    fn rust_mentions_cargo() {
        assert!(get_claude_md(ProjectType::Rust)
            .to_lowercase()
            .contains("cargo"));
    }

    #[test]
    fn go_mentions_go_build() {
        assert!(get_claude_md(ProjectType::Go).contains("go build"));
    }

    #[test]
    fn python_mentions_pytest() {
        assert!(get_claude_md(ProjectType::Python).contains("pytest"));
    }

    #[test]
    fn generic_has_todo_placeholders() {
        assert!(get_claude_md(ProjectType::Generic).contains("TODO"));
    }

    #[test]
    fn all_templates_non_empty() {
        for t in [
            ProjectType::TypescriptBun,
            ProjectType::TypescriptNode,
            ProjectType::Rust,
            ProjectType::Go,
            ProjectType::Python,
            ProjectType::Generic,
        ] {
            assert!(!get_claude_md(t).is_empty(), "{} empty", t.as_str());
        }
    }

    #[test]
    fn plan_command_has_arguments_placeholder() {
        assert!(get_plan_command().contains("$ARGUMENTS"));
    }

    #[test]
    fn review_command_has_arguments_placeholder() {
        assert!(get_review_command().contains("$ARGUMENTS"));
    }
}
