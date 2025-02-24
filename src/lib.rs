pub mod routes;
pub mod services;
pub mod settings;
pub mod startup;
pub mod telemetry;
pub mod types;
pub mod utils;

#[cfg(test)]
pub mod test_utils;

pub static ENV: once_cell::sync::Lazy<minijinja::Environment<'static>> =
    once_cell::sync::Lazy::new(|| {
        let mut env = minijinja::Environment::new();
        env.set_loader(minijinja::path_loader("templates"));
        env
    });
