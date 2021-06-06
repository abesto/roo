#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Command {
    VerbNoArgs { verb: String },
    VerbDirect { verb: String, direct: String },
}

impl Command {
    pub fn verb(&self) -> &str {
        match self {
            Command::VerbNoArgs { verb } => verb,
            Command::VerbDirect {
                verb,
                direct: _direct,
            } => verb,
        }
    }

    pub fn to_args(&self) -> Vec<String> {
        match self {
            Command::VerbNoArgs { verb: _verb } => vec![],
            Command::VerbDirect {
                verb: _verb,
                direct,
            } => vec![direct.clone()],
        }
    }
}

pub fn parse_command(input: &str) -> Option<Command> {
    let processed = {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        if let Some(say_msg) = trimmed.strip_prefix('"') {
            format!("say {}", say_msg)
        } else if let Some(emote_msg) = trimmed.strip_prefix(':') {
            format!("emote {}", emote_msg)
        } else {
            trimmed.to_string()
        }
    };

    let words: Vec<_> = processed.split_whitespace().collect();
    if words.len() == 1 {
        Some(Command::VerbNoArgs {
            verb: processed.to_string(),
        })
    } else if words.len() > 1 {
        let verb = words[0];
        Some(Command::VerbDirect {
            verb: verb.to_string(),
            direct: processed.strip_prefix(verb)?.trim().to_string(),
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plain_verb() {
        assert_eq!(
            Some(Command::VerbNoArgs {
                verb: "look".to_string()
            }),
            parse_command("  look ")
        );
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(None, parse_command("   "));
    }

    #[test]
    fn test_say() {
        assert_eq!(
            Some(Command::VerbDirect {
                verb: "say".to_string(),
                direct: "hello hi how is it going".to_string()
            }),
            parse_command("say hello hi how is it going")
        );
    }

    #[test]
    fn test_shortcuts() {
        assert_eq!(
            Some(Command::VerbDirect {
                verb: "say".to_string(),
                direct: "test say message".to_string()
            }),
            parse_command("\"test say message")
        );

        assert_eq!(
            Some(Command::VerbDirect {
                verb: "emote".to_string(),
                direct: "test emote message".to_string()
            }),
            parse_command(":test emote message")
        );
    }
}
