use std::convert::TryFrom;

#[derive(Clone, Debug)]
pub enum VerbSignature {
    NoArgs { name: String },
}

impl VerbSignature {
    pub fn name(&self) -> &str {
        match self {
            VerbSignature::NoArgs { name } => &name,
        }
    }
}

impl<S> TryFrom<&Vec<S>> for VerbSignature
where
    S: ToString,
{
    type Error = String;

    fn try_from(value: &Vec<S>) -> Result<Self, Self::Error> {
        if value.len() != 1 {
            return Err("Expected exactly one item in signature".to_string());
        }
        Ok(Self::NoArgs {
            name: value[0].to_string(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Verb {
    pub(super) signature: VerbSignature,
    pub code: String,
}

impl Verb {
    #[must_use]
    fn new(signature: VerbSignature) -> Self {
        Self {
            signature,
            code: String::new(),
        }
    }

    pub fn name(&self) -> &str {
        self.signature.name()
    }
}

impl<S> TryFrom<&Vec<S>> for Verb
where
    S: ToString,
{
    type Error = String;

    fn try_from(value: &Vec<S>) -> Result<Self, Self::Error> {
        Ok(Self::new(VerbSignature::try_from(value)?))
    }
}
