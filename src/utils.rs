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

/**
- Basically if I have a list and I search for `"run"`, I would get results like `["run", "runner", "running"]`
- This uses binary search and then moves adjacent while it still matches the condition
*/
pub fn find_all_extended_words(strings: &Vec<String>, word: &str) -> Option<Vec<String>> {
    let index = strings
        .binary_search_by(|s| {
            if s.starts_with(word) {
                std::cmp::Ordering::Equal
            } else if s.as_str() < word {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        })
        .ok()?;
    let mut results = vec![strings.get(index).expect("Already verified").clone()];

    let mut left_index = index - 1;
    while let Some(left) = strings.get(left_index) {
        if !left.as_str().starts_with(word) {
            break;
        }
        results.push(left.clone());
        if left_index == 0 {
            break;
        }
        left_index -= 1;
    }

    let mut right_index = index + 1;
    while let Some(right) = strings.get(right_index) {
        if !right.as_str().starts_with(word) {
            break;
        }
        results.push(right.clone());
        right_index += 1;
    }

    Some(results)
}
