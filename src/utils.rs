use std::{io::Write, time::Instant};

pub struct Timer(Instant);

impl Timer {
    pub fn new() -> Self {
        Self(Instant::now())
    }
    pub fn reset(&mut self) {
        self.0 = Instant::now();
    }
    pub fn print(&mut self, msg: &str) {
        println!("[{}ms] {}", self.0.elapsed().as_millis(), msg);
        self.reset();
    }
}

pub fn prompt(input_prompt: &str) -> String {
    let mut input_line = String::new();
    print!("{input_prompt}");
    std::io::stdout().flush().unwrap();
    _ = std::io::stdin().read_line(&mut input_line);
    input_line.trim().to_string()
}

/**
```no_run
// so I can rewrite
let mut config_path = get_config_path()?;
config_path.push("config.toml");
Some(config_path)
// as either this
Some(get_config_path()?.mutated(|config| config.push("config.toml")))
// or this
get_config_path().map(|config| config.mutated(|config| config.push("config.toml")))
```
*/
pub trait Mutated: Sized {
    fn mutated(mut self, func: impl Fn(&mut Self)) -> Self {
        func(&mut self);
        self
    }
}

impl<T> Mutated for T {}
