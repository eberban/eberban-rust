use easy_ext::ext;

use crate::{Map, MapExt, Parser};

pub type Discard<S, A, AO> = Map<S, A, AO, fn(AO) -> (), ()>;

/// Provide [`discard`](Self::discard) method to all [`Parser`].
#[ext(DiscardExt)]
pub impl<Self_, S, I, AO> Self_
where
    Self: Sized + Parser<S, I, AO>,
{
    /// Parse from `Self`, then discard its output.
    ///
    /// Rollbacks if `Self` fails to parse, and stream will only progress with
    /// what `Self` parsed.
    fn discard(self) -> Discard<S, Self, AO> {
        self.map(std::mem::drop as fn(_) -> ())
    }
}
