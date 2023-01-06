use {easy_ext::ext, framework::Span};

pub mod morpho;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub span: Span,
    pub text: String,
}

#[ext(SpannedErrorExt)]
pub impl Span {
    fn error(self, text: impl ToString) -> Error {
        Error {
            span: self,
            text: text.to_string(),
        }
    }
}
