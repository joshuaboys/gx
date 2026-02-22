import type { ProjectType } from "./detect.ts";

export function getClaudeMd(type: ProjectType): string {
  switch (type) {
    case "typescript-bun":
      return TYPESCRIPT_BUN_TEMPLATE;
    case "typescript-node":
      return TYPESCRIPT_NODE_TEMPLATE;
    case "rust":
      return RUST_TEMPLATE;
    case "go":
      return GO_TEMPLATE;
    case "python":
      return PYTHON_TEMPLATE;
    case "generic":
      return GENERIC_TEMPLATE;
  }
}

export function getPlanCommand(): string {
  return PLAN_COMMAND;
}

export function getReviewCommand(): string {
  return REVIEW_COMMAND;
}

// -- Templates --

const TYPESCRIPT_BUN_TEMPLATE = `# Project Context

## Stack
- Runtime: Bun
- Language: TypeScript

## Commands
- Run: \`bun run <file>\`
- Test: \`bun test\`
- Build: \`bun build <entry>\`
- Install: \`bun install\`
- Lint/Format: \`bunx biome check --write .\`

## Conventions
- Use \`bun:test\` for testing (describe/test/expect)
- Use \`Bun.file()\` / \`Bun.write()\` for file I/O
- Use \`Bun.serve()\` for HTTP servers
- Bun loads .env automatically — no dotenv needed
`;

const TYPESCRIPT_NODE_TEMPLATE = `# Project Context

## Stack
- Runtime: Node.js
- Language: TypeScript

## Commands
- Run: \`npx tsx <file>\`
- Test: \`npm test\`
- Build: \`npm run build\`
- Install: \`npm install\`
- Lint/Format: \`npx eslint . && npx prettier --write .\`

## Conventions
- Follow existing test framework conventions in the project
- Use async/await over raw promises
- Prefer named exports
`;

const RUST_TEMPLATE = `# Project Context

## Stack
- Language: Rust

## Commands
- Build: \`cargo build\`
- Test: \`cargo test\`
- Lint: \`cargo clippy -- -D warnings\`
- Format: \`cargo fmt\`
- Run: \`cargo run\`

## Conventions
- Run \`cargo clippy\` before committing
- Run \`cargo fmt\` to format code
- Handle errors with Result — avoid unwrap in library code
- Write doc comments for public items
`;

const GO_TEMPLATE = `# Project Context

## Stack
- Language: Go

## Commands
- Build: \`go build ./...\`
- Test: \`go test ./...\`
- Vet: \`go vet ./...\`
- Format: \`gofmt -w .\`

## Conventions
- Run \`go vet\` before committing
- Use \`gofmt\` for formatting
- Follow standard Go project layout
- Handle all errors — never ignore returned errors
`;

const PYTHON_TEMPLATE = `# Project Context

## Stack
- Language: Python

## Commands
- Test: \`pytest\`
- Lint: \`ruff check .\`
- Format: \`ruff format .\`
- Type check: \`mypy .\`

## Conventions
- Use type hints for function signatures
- Use pytest for testing
- Use ruff or black for formatting
- Follow PEP 8 style guidelines
`;

const GENERIC_TEMPLATE = `# Project Context

## Stack
- TODO: Describe the project stack here

## Commands
- TODO: Add common commands (build, test, lint, run)

## Conventions
- TODO: Add project-specific conventions and guidelines
`;

const PLAN_COMMAND = `Review the codebase and create a detailed implementation plan for:

$ARGUMENTS

Include:
1. Files to create or modify
2. Key design decisions
3. Testing approach
4. Potential edge cases
`;

const REVIEW_COMMAND = `Review the following code changes for bugs, style issues, and improvements:

$ARGUMENTS

Check for:
1. Correctness and edge cases
2. Error handling
3. Naming and readability
4. Performance concerns
5. Security issues
`;
