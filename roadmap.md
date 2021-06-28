* Migrate existing DatabaseProxy methods to end-to-end error reporting (and add test coverage)
* Do *not* create properties on demand, implement `add_property` and `delete_property` instead
* parent, children, etc should not be accessible as properties, implement `parent()` etc