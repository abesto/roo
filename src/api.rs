use rhai::Engine;

// The built-in functions available for ROO scripts

pub fn register_api(engine: &mut Engine) {
    engine.register_fn("echo", |s: &str| s.to_string());
}