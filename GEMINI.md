# Instructions for AI Coding Agents

First read `README.md` to get an overview of what the project does.

Only do the tasks described in the following sections if explicitly requested to.

**Do not run linting or compilation steps when writing code.** Your task is to create
the structure and it's okay to have details not yet implemented.

## Coding Guidelines
* Rust import statemnts should be one crate per statement. Importing multiple objects from the same create should be done in the same statement.
  * Good: `use anyhow::{anyhow, Error, Result}`
  * Bad: the above imports on separate lines/statements for each imported object
* Rust import statements should use separate lines for imports from different modules originating from this project.

