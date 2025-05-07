use core::iter::Iterator;

/// Represents a part of an SCPI command, such as "STATus" in "STATus:EVENt?".
/// 
/// Each part has both a short form (uppercase letters only) and a long form (complete word).
/// SCPI allows using either form in commands.
/// 
/// For example, "STATus" can be written as either "STAT" (short form) or "STATUS" (long form).
#[derive(Debug, Clone, PartialEq)]
pub struct CommandPart {
    /// Whether this command part is optional.
    pub optional: bool,
    /// The short form of the command part.
    pub short: String,
    /// The long form of the command part.
    pub long: String,
}

/// Represents a complete SCPI command with all its parts.
/// 
/// An SCPI command consists of multiple parts separated by colons, for example:
/// "SYSTem:ERRor:NEXT?" is a command with three parts and is a query (ends with '?').
/// 
/// The command can also have optional parts, indicated by square brackets, like:
/// "[STATus]:EVENt?" where "STATus" is optional.
#[derive(Debug, Clone)]
pub struct Command {
    /// The parts of the command name.
    pub parts: Vec<CommandPart>,
    /// Whether the command is a query, i.e. ends with a question mark.
    query: bool,
}

/// Represents a specific path through the command tree, with each string
/// representing either the short or long form of a command part.
pub type CommandPath = Vec<String>;

impl TryFrom<&str> for Command {
    type Error = Box<dyn std::error::Error>;

    /// Parses a command string into a Command structure.
    /// 
    /// # Arguments
    /// * `value` - The SCPI command string (e.g., "SYSTem:ERRor?" or "[STATus]:EVENt?")
    /// 
    /// # Returns
    /// * `Ok(Command)` - Successfully parsed command
    /// * `Err` - If the command string is invalid
    /// 
    /// # Examples
    /// ```
    /// use microscpi_macros::command::Command;
    /// let cmd = Command::try_from("SYSTem:ERRor?").unwrap();
    /// assert!(cmd.is_query());
    /// ```
    fn try_from(mut value: &str) -> Result<Self, Self::Error> {
        let mut parts = Vec::new();
        let mut query = false;

        // Check if the command is a query (ends with '?')
        if let Some(prefix) = value.strip_suffix('?') {
            value = prefix;
            query = true;
        }

        // Process each part of the command (separated by colons)
        for part in value.split(':').map(str::trim) {
            if part.is_empty() {
                continue;
            }

            // Check if this part is optional (enclosed in square brackets)
            let (part, optional) = if part.starts_with('[') && part.ends_with(']') {
                (&part[1..part.len() - 1], true)
            } else {
                (part, false)
            };

            // The short form consists of only the uppercase letters
            let short = part.chars().filter(|c| !c.is_lowercase()).collect();
            // The long form is the entire part in uppercase
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
    /// Returns whether this command is a query (ends with a question mark).
    pub fn is_query(&self) -> bool {
        self.query
    }

    /// Returns the canonical (long-form) representation of this command.
    /// 
    /// This is the complete command with all parts in their long form,
    /// separated by colons, and with a question mark at the end if it's a query.
    pub fn canonical_path(&self) -> String {
        // Build the path using all long forms
        let path = self
            .parts
            .iter()
            .fold(String::new(), |a, b| {
                if a.is_empty() {
                    b.long.clone()
                } else {
                    a + ":" + &b.long
                }
            });

        if self.query { path + "?" } else { path }
    }

    /// Generates all valid paths for this command.
    /// 
    /// Since SCPI commands can have optional parts and each part can be
    /// specified in either short or long form, this method generates
    /// all possible valid combinations.
    /// 
    /// # Returns
    /// A vector of all valid command paths
    pub fn paths(&self) -> Vec<CommandPath> {
        let mut paths: Vec<CommandPath> = vec![vec![]];

        for part in &self.parts {
            let mut new_paths: Vec<CommandPath> = Vec::new();

            for path in &mut paths {
                // Add the long form
                let mut long_path = path.clone();
                long_path.push(part.long.clone());
                new_paths.push(long_path);

                // Add the short form if it's different from the long form
                if part.short != part.long {
                    let mut short_path = path.clone();
                    short_path.push(part.short.clone());
                    new_paths.push(short_path);
                }

                // If this part is optional, add a path without it
                if part.optional {
                    new_paths.push(path.clone());
                }
            }

            paths = new_paths;
        }

        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_paths() {
        let cmd = Command::try_from("[STATus]:TIMe?").unwrap();
        let paths: Vec<CommandPath> = cmd.paths();
        
        // Ensure we generate all valid path combinations:
        // 1. With STATUS (long) + TIME (long)
        assert!(paths.iter().any(|p| p.as_ref() == vec!["STATUS", "TIME"]));
        // 2. With STATUS (long) + TIM (short)
        assert!(paths.iter().any(|p| p.as_ref() == vec!["STATUS", "TIM"]));
        // 3. With STAT (short) + TIME (long)
        assert!(paths.iter().any(|p| p.as_ref() == vec!["STAT", "TIME"]));
        // 4. With STAT (short) + TIM (short)
        assert!(paths.iter().any(|p| p.as_ref() == vec!["STAT", "TIM"]));
        // 5. With just TIME (long) - since STATUS is optional
        assert!(paths.iter().any(|p| p.as_ref() == vec!["TIME"]));
        // 6. With just TIM (short) - since STATUS is optional
        assert!(paths.iter().any(|p| p.as_ref() == vec!["TIM"]));
    }

    #[test]
    fn test_is_query() {
        let cmd = Command::try_from("SYSTem:ERRor?").unwrap();
        assert!(cmd.is_query());

        let cmd = Command::try_from("SYSTem:ERRor").unwrap();
        assert!(!cmd.is_query());
    }

    #[test]
    fn test_canonical_path() {
        let cmd = Command::try_from("[STATus]:TIMe?").unwrap();
        assert_eq!(cmd.canonical_path(), "STATUS:TIME?");

        let cmd = Command::try_from("SYSTem:ERRor:NEXT").unwrap();
        assert_eq!(cmd.canonical_path(), "SYSTEM:ERROR:NEXT");
    }
}
