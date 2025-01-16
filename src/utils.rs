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
