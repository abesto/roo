* Rust integration is amazing
* Nice to have: custom implementation for variables with symbol prefixes (#0, $nothing)
* Spread assignment: cool that it can be implemented with custom syntax. Maybe add to language?
* Custom error types are not useful because their value gets lost when printed via the `Display` implementation of `Dynamic`, workaround is `impl From<Dynamic> for Box<EvalAltResult>` creating an object map, but we lose type checking / custom methods on it