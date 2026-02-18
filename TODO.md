# flut — Implementation TODO

## Phase 1: Foundation

- [x] Project setup (Cargo.toml, module structure)
- [x] Basic REPL loop with reedline
- [x] Custom prompt

## Phase 2: Lexer

- [x] Token types (identifiers, pipes, operators, strings, numbers, keywords)
- [x] Span tracking on all tokens (byte offsets for error reporting)
- [x] Newline handling (statement termination, continuation after `|`)
- [x] Lexer error reporting with spans
- [x] Lexer tests

## Phase 3: Parser

- [x] AST types (commands, pipelines, let-bindings, expressions)
- [ ] Parse simple commands with arguments (`ls ./src`)
- [ ] Parse pipelines (`ls | where ext == "rs"`)
- [ ] Parse let-bindings (`let x = | where ext == "rs"`)
- [ ] Parse closures / blocks (`{ curl $.url }`)
- [ ] Parse effect syntax (`with retry(3) parallel(4)`)
- [ ] Parser error reporting with spans
- [ ] Parser tests

## Phase 4: Error Reporting

- [ ] Choose diagnostic crate (ariadne, miette, codespan-reporting)
- [ ] Integrate span-based errors from lexer
- [ ] Integrate span-based errors from parser
- [ ] Helpful error messages with suggestions

## Phase 5: Evaluator

- [ ] Value types (string, int, float, bool, list, record, table, null)
- [ ] Execute simple commands
- [ ] Pipeline execution (pipe stage output to next stage input)
- [ ] Variable resolution (`$x`)
- [ ] Environment / scope

## Phase 6: Built-in Commands

- [ ] `ls` — list directory, return structured records
- [ ] `where` — filter rows by condition
- [ ] `sort-by` — sort by column
- [ ] `take` / `skip` — slice rows
- [ ] `each` — map over rows
- [ ] `get` — access column/field
- [ ] `open` — read file into structured data
- [ ] External command execution (fallback to system binaries)

## Phase 7: Pipeline Composition

- [ ] Pipelines as first-class values
- [ ] Pipeline variables (`let p = | where ...`)
- [ ] Pipeline application (`data | $p`)
- [ ] Point-free composition

## Phase 8: History & Completion

- [ ] File-backed history (reedline built-in)
- [ ] Tab completion for built-in commands
- [ ] Tab completion for file paths
- [ ] Syntax highlighting via reedline

## Phase 9: Observability

- [ ] Stage tracing (timing, input/output shapes)
- [ ] `inspect` command (view stage-level data)
- [ ] Data snapshots at each stage

## Phase 10: Effects

- [ ] Effect middleware system
- [ ] `retry(n)`
- [ ] `timeout(duration)`
- [ ] `parallel(n)`
- [ ] `cache`

## Future

- [ ] Script file execution
- [ ] SQLite-backed history
- [ ] Vector search over history (semantic recall)
- [ ] Custom prompt scripting
- [ ] Plugin system
