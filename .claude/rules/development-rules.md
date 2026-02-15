# Development Rules

**IMPORTANT:** Analyze the skills catalog and activate the skills that are needed for the task during the process.
**IMPORTANT:** You ALWAYS follow these principles: **YAGNI (You Aren't Gonna Need It) - KISS (Keep It Simple, Stupid) - DRY (Don't Repeat Yourself)**

## General
- **File Naming**: Use kebab-case for file names with a meaningful name that describes the purpose of the file, doesn't matter if the file name is long, just make sure when LLMs read the file names while using Grep or other tools, they can understand the purpose of the file right away without reading the file content.
- **File Size Management**: Keep individual code files under 200 lines for optimal context management
  - Split large files into smaller, focused components/modules
  - Use composition over inheritance for complex widgets
  - Extract utility functions into separate modules
  - Create dedicated service classes for business logic
- When looking for docs, activate `docs-seeker` skill (`context7` reference) for exploring latest docs.
- Use `gh` bash command to interact with Github features if needed
- Use `psql` bash command to query Postgres database for debugging if needed
- Use `ai-multimodal` skill for describing details of images, videos, documents, etc. if needed
- Use `ai-multimodal` skill and `imagemagick` skill for generating and editing images, videos, documents, etc. if needed
- Use `sequential-thinking` and `debugging` skills for sequential thinking, analyzing code, debugging, etc. if needed
- **[IMPORTANT]** Follow the codebase structure and code standards in `./docs` during implementation.
- **[IMPORTANT]** Do not just simulate the implementation or mocking them, always implement the real code.

## Code Quality Guidelines
- Read and follow codebase structure and code standards in `./docs`
- Don't be too harsh on code linting, but **make sure there are no syntax errors and code are compilable**
- Prioritize functionality and readability over strict style enforcement and code formatting
- Use reasonable code quality standards that enhance developer productivity
- Use try catch error handling & cover security standards
- Use `code-reviewer` agent to review code after every implementation

## Pre-commit/Push Rules
- Run linting before commit
- Run tests before push (DO NOT ignore failed tests just to pass the build or github actions)
- Keep commits focused on the actual code changes
- **DO NOT** commit and push any confidential information (such as dotenv files, API keys, database credentials, etc.) to git repository!
- Create clean, professional commit messages without AI references. Use conventional commit format.

## Code Implementation
- Write clean, readable, and maintainable code
- Follow established architectural patterns
- Implement features according to specifications
- Handle edge cases and error scenarios
- **DO NOT** create new enhanced files, update to the existing files directly.

## Testing Requirements (MANDATORY)

**⛔ NO TEST = NO DONE** - Every feature MUST have corresponding tests verified in the RAM environment before considered complete.

### Core Protocol (WSL/RAM)
- **DOCKER ONLY:** NEVER run `cargo`, `pytest`, or `npm` on the host. Host execution will freeze the system due to WSL I/O issues.
- **MIGRATIONS:** Always run migrations inside the container before linting/testing.
- **CLEANUP:** Use `./run-tests.sh --clean` if RAM volumes are full or corrupted.

### General Testing Rules
- Every new feature, bug fix, or refactor MUST include tests.
- Tests must pass before marking any task as complete.
- **[CRITICAL]** If tests fail, you MUST use `./scripts/debug-logs.sh --errors` to investigate. Do not guess the cause.
- Use `cargo nextest` inside Docker for faster, isolated Rust testing.

### UI/Frontend Testing Standards
- **ALWAYS use `data-testid` attributes** for element selection in tests
- Format: `data-testid="component-name-element-type"` (kebab-case)
- Examples:
  - `data-testid="login-form-submit-button"`
  - `data-testid="user-profile-avatar"`
  - `data-testid="device-list-toggle-switch"`
- **DO NOT** use class names, tag names, or text content for test selectors
- Add `data-testid` to interactive elements: buttons, inputs, links, toggles
- Add `data-testid` to dynamic content areas: lists, cards, modals

### Test Coverage Requirements
- Unit tests: Core logic, utility functions, hooks
- Integration tests: API endpoints, database operations
- E2E tests: Critical user flows (login, main features)
- Minimum coverage target: 80%

### Definition of Done Checklist
- [ ] Feature implemented and working
- [ ] Unit tests written and passing
- [ ] `data-testid` attributes added to UI components
- [ ] Integration/E2E tests if applicable
- [ ] All tests pass via `./run-tests.sh`