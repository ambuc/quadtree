# Development notes

## Hooks

There is a pre-commit hook inside `/development/hooks/pre-commit`. Install it by
copying it to `.git/hooks/pre-commit`. This will run a suite of commands
(format, test, etc.) automatically before a commit.

NB: git pre-commit hooks can be invoked manually with `bash .git/hooks/pre-commit`.
