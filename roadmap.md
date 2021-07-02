* Migrate existing DatabaseProxy methods to end-to-end error reporting (and add test coverage)
* `anyhow`
* Do *not* create properties on demand, implement `add_property` and `delete_property` instead
* add object uuid to verb code chunk name for better stack traces
* parent, children, etc should not be accessible as properties, implement `parent()` etc
* drop `DatabaseProxy::has_verb_with_name`, implement `verbs()`, reimplement `S.object_utils.has_verb` on top of it
* move `resolve_verb` into (inline?) Lua?
* allow mutating string properties
* drop all `.map(|_| LuaValue::Nil)` once https://github.com/khvzak/mlua/pull/60 is resolved
* full verbspec support
...
* permissions