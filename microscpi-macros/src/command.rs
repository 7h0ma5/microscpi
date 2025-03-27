use core::iter::Iterator;

#[derive(Debug, Clone, PartialEq)]
pub struct CommandPart {
    pub optional: bool,
    pub short: String,
    pub long: String,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub parts: Vec<CommandPart>,
    query: bool,
}

pub type CommandPath = Vec<String>;

impl TryFrom<&str> for Command {
    type Error = Box<dyn std::error::Error>;

    fn try_from(mut value: &str) -> Result<Self, Self::Error> {
        let mut parts = Vec::new();
        let mut query = false;

        if let Some(prefix) = value.strip_suffix('?') {
            value = prefix;
            query = true;
        }

        for part in value.split(':').map(str::trim) {
            if part.is_empty() {
                continue;
            }

            let (part, optional) = if part.starts_with('[') && part.ends_with(']') {
                (&part[1..part.len() - 1], true)
            } else {
                (part, false)
            };

            let short = part.chars().filter(|c| !c.is_lowercase()).collect();
            let long = part.to_uppercase();

            parts.push(CommandPart {
                optional,
                short,
                long,
            });
        }

        Ok(Command { parts, query })
    }
}

impl Command {
    pub fn is_query(&self) -> bool {
        self.query
    }

    pub fn paths(&self) -> Vec<CommandPath> {
        let mut paths: Vec<CommandPath> = vec![vec![]];

        for part in &self.parts {
            let mut new_paths: Vec<CommandPath> = Vec::new();

            for path in &mut paths {
                let mut long_path = path.clone();
                long_path.push(part.long.clone());
                new_paths.push(long_path);

                if part.short != part.long {
                    let mut short_path = path.clone();
                    short_path.push(part.short.clone());
                    new_paths.push(short_path);
                }

                if part.optional {
                    new_paths.push(path.clone());
                }
            }

            paths = new_paths;
        }

        paths
    }
}

#[test]
pub fn test_all_paths() {
    let cmd = Command::try_from("[STATus]:TIMe?").unwrap();
    let paths: Vec<CommandPath> = cmd.paths();
    println!("{:?}", cmd);
    println!("{:?}", paths);
    assert!(paths.iter().any(|p| p.as_ref() == vec!["STAT", "TIM"]));
    assert!(paths.iter().any(|p| p.as_ref() == vec!["STATUS", "TIM"]));
    assert!(paths.iter().any(|p| p.as_ref() == vec!["STATUS", "TIME"]));
    assert!(paths.iter().any(|p| p.as_ref() == vec!["TIM"]));
    assert!(paths.iter().any(|p| p.as_ref() == vec!["TIME"]));
}
