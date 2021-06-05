#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Command {
    Verb { verb: String },
}

impl Command {
    pub fn verb(&self) -> &str {
        match self {
            Command::Verb { verb } => verb,
        }
    }
}

pub fn parse_command(input: &str) -> Option<Command> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(Command::Verb {
        verb: trimmed.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plain_verb() {
        assert_eq!(
            Some(Command::Verb {
                verb: "look".to_string()
            }),
            parse_command("  look ")
        );
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(None, parse_command("   "));
    }
}
