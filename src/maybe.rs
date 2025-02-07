#[diagnostic::on_unimplemented(
    message = "this is a helper for [Option, Result].",
    label = "you might want collect_array_checked"
)]
#[doc(hidden)]
pub trait Maybe {
    type Unwrap;
    type Or;
    fn asr(self) -> Result<Self::Unwrap, Self::Or>;
}
impl<T> Maybe for Option<T> {
    type Unwrap = T;
    type Or = ();
    fn asr(self) -> Result<Self::Unwrap, Self::Or> {
        self.ok_or(())
    }
}
impl<T, E> Maybe for Result<T, E> {
    type Unwrap = T;
    type Or = E;
    fn asr(self) -> Result<Self::Unwrap, Self::Or> {
        self
    }
}
