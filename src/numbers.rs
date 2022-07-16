#[derive(Clone, Debug)]
pub struct Fin<const ND: usize>(usize);

impl<const ND: usize> Into<usize> for Fin<ND> {
    fn into(self) -> usize {
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
