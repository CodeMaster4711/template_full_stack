CODING AGENT INSTRUCTIONS

CORE RULES
- Write production-grade, clean code only
- No comments, no docstrings, no inline explanations
- No emojis anywhere — not in logs, errors, output, variable names, or strings
- No placeholder code, no TODOs, no FIXMEs
- No dead code, no unused imports, no unused variables
- No console.log/print debugging left in output

CODE STYLE
- Follow the language-native convention strictly (PEP8, Google Style, Airbnb, etc.)
- Meaningful, concise names — no abbreviations unless universally understood (e.g. id, url, db)
- Single responsibility per function and class
- Max function length: 20–30 lines; extract if longer
- Prefer pure functions over stateful logic where possible
- Avoid deeply nested logic — early returns, guard clauses
- Flat is better than nested

EFFICIENCY
- Minimize allocations, copies, and redundant computations
- Prefer built-ins and standard library over third-party where equivalent
- Choose the right data structure for the access pattern
- Avoid over-engineering — simplest correct solution wins

LOGGING
- Use structured logging (key=value or JSON), no freeform strings with emojis or decorators
- Log levels: DEBUG, INFO, WARN, ERROR — nothing custom
- Log only what is operationally relevant
- Format: [LEVEL] message key=value

ERROR HANDLING
- Fail fast and explicitly
- Never swallow exceptions silently
- Use specific exception types, not bare except/catch
- Propagate errors to the appropriate layer; don't handle where you can't recover

OUTPUT FORMAT
- Return only the final code
- No preamble, no explanation, no markdown unless the target format requires it
- If multiple files, output each with a clear filename header and nothing else

COMMIT CONVENTIONS & WORKFLOW
- Automatic Suggestions: Proactively suggest a git commit message after completing any significant block or logical step, even if not explicitly asked.
- Format: Follow Conventional Commits specification (e.g., feat:, fix:, chore:, docs:, refactor:, style:, test:).
- Style: Single, concise English sentence in the imperative mood. No periods at the end.
- Flow Control:
    - For single logical steps: Provide code and commit message immediately.
    - For large, multi-step tasks: Provide the code for the major block, suggest the commit message, and WAIT for approval before proceeding to the next major step.
