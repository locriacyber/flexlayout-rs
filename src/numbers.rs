#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fin<const ND: usize>(usize);

impl<const ND: usize> Fin<ND> {
    pub fn into_usize(self) -> usize {
        self.0
    }
}

impl<const ND: usize> TryFrom<usize> for Fin<ND> {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value < ND {
            Ok(Self(value))
        } else {
            Err(())
        }
    }
}
