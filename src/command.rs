use uuid::Uuid;

use crate::database::Database;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct VerbObject {
    str: String,
    obj: Option<Uuid>,
}

impl VerbObject {
    #[must_use]
    fn new<S: ToString>(str: S, obj: Option<Uuid>) -> Self {
        Self {
            str: str.to_string(),
            obj,
        }
    }

    #[must_use]
    fn resolve<S: ToString>(str: S, player: &Uuid, db: &Database) -> Self {
        let obj = db
            .resolve_object(player, &str.to_string())
            .map(|o| o.uuid().clone());
        Self::new(str, obj)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ParsedCommand {
    VerbNoArgs { verb: String },
    VerbDirect { verb: String, direct: VerbObject },
    // TODO move special commands in here and maybe execution as well
}

impl ParsedCommand {
    #[must_use]
    fn verb_no_args<S: ToString>(verb: S) -> Self {
        Self::VerbNoArgs {
            verb: verb.to_string(),
        }
    }

    #[must_use]
    fn verb_direct<V: ToString, D: ToString>(
        verb: V,
        direct: D,
        player: &Uuid,
        db: &Database,
    ) -> Self {
        Self::VerbDirect {
            verb: verb.to_string(),
            direct: VerbObject::resolve(direct, player, db),
        }
    }

    fn dobjstr(&self) -> String {
        match &self {
            Self::VerbNoArgs { verb: _verb } => String::new(),
            Self::VerbDirect {
                verb: _verb,
                direct,
            } => direct.str.clone(),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Command {
    argstr: String,
    args: Vec<String>,
    parsed: ParsedCommand,
}

impl Command {
    #[must_use]
    pub fn new<A: ToString, X: ToString, AS: IntoIterator<Item = X>>(
        argstr: A,
        args: AS,
        parsed: ParsedCommand,
    ) -> Self {
        Self {
            argstr: argstr.to_string(),
            args: args.into_iter().map(|a| a.to_string()).collect(),
            parsed,
        }
    }
    pub fn verb(&self) -> &str {
        match &self.parsed {
            ParsedCommand::VerbNoArgs { verb } => verb,
            ParsedCommand::VerbDirect {
                verb,
                direct: _direct,
            } => verb,
        }
    }

    pub fn args(&self) -> &Vec<String> {
        &self.args
    }

    pub fn argstr(&self) -> &String {
        &self.argstr
    }

    pub fn dobjstr(&self) -> String {
        self.parsed.dobjstr()
    }

    pub fn parsed(&self) -> &ParsedCommand {
        &self.parsed
    }

    pub fn parse<S: ToString>(input: S, player: &Uuid, db: &Database) -> Option<Command> {
        let processed = {
            let trimmed = input.to_string().trim().to_string();
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

        // TODO this splitting is clowny AF
        let parts: Vec<String> = processed.splitn(2, ' ').map(ToString::to_string).collect();
        let verb = parts[0].clone();
        let args = parts[1..].to_vec();
        let argstr = processed[verb.len()..].trim().to_string();

        let parsed = match parts.as_slice() {
            [verb] => Some(ParsedCommand::verb_no_args(verb)),
            [verb, direct] => Some(ParsedCommand::verb_direct(verb, direct, player, db)),
            _ => None,
        }?;

        Some(Self {
            args,
            argstr,
            parsed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plain_verb() {
        let mut db = Database::new();
        let player = db.create();
        assert_eq!(
            Some(Command::new(
                "",
                <Vec<String>>::new(),
                ParsedCommand::verb_no_args("look")
            )),
            Command::parse("  look  ", &player, &db)
        );
    }

    #[test]
    fn test_empty_input() {
        let mut db = Database::new();
        let player = db.create();
        assert_eq!(None, Command::parse("   ", &player, &db));
    }

    #[test]
    fn test_verb_direct() {
        let mut db = Database::new();
        let player = db.create();

        // Object not found
        assert_eq!(
            ParsedCommand::VerbDirect {
                verb: "direct1".to_string(),
                direct: VerbObject {
                    str: "foobar".to_string(),
                    obj: None
                }
            },
            ParsedCommand::verb_direct("direct1", "foobar", &player, &db)
        );

        // Object by UUID in inventory
        let object = db.create();
        db.move_object(&object, &player).unwrap();
        assert_eq!(
            ParsedCommand::VerbDirect {
                verb: "direct2".to_string(),
                direct: VerbObject {
                    str: object.to_string(),
                    obj: Some(object.clone())
                }
            },
            ParsedCommand::verb_direct("direct2", object.to_string(), &player, &db)
        );
    }

    #[test]
    fn test_say() {
        let mut db = Database::new();
        let player = db.create();

        assert_eq!(
            Some(Command::new(
                "hi hello  how is",
                vec!["hi hello  how is"],
                ParsedCommand::verb_direct("say", "hi hello  how is", &player, &db)
            )),
            Command::parse("say hi hello  how is", &player, &db)
        );
    }

    #[test]
    fn test_shortcuts() {
        let mut db = Database::new();
        let player = db.create();

        assert_eq!(
            Some(Command::new(
                "test say message",
                vec!["test say message"],
                ParsedCommand::verb_direct("say", "test say message", &player, &db)
            )),
            Command::parse("\"test say message", &player, &db)
        );

        assert_eq!(
            Some(Command::new(
                "test emote message",
                vec!["test emote message"],
                ParsedCommand::verb_direct("emote", "test emote message", &player, &db)
            )),
            Command::parse(":test emote message", &player, &db)
        );
    }
}
