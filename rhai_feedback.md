* Rust integration is amazing
* Nice to have: custom implementation for variables with symbol prefixes (#0, $nothing)
  * Related: would be nice to be able to parse #-1
  * A possible cool implementation would be custom unary operators?
* Spread assignment: cool that it can be implemented with custom syntax. Maybe add to language?
* Custom error types are not useful because their value gets lost when printed via the `Display` implementation of `Dynamic`, workaround is `impl From<Dynamic> for Box<EvalAltResult>` creating an object map, but we lose type checking / custom methods on it
* Function aliases would be nice. Manually duplicating for a new name gets tedious in the presence of heavy overloading.
* Would be nice to be able to call registered functions from other registered functions via the Rhai dispatch process (especially given overloading)