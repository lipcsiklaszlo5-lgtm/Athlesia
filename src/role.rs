#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Role {
    id: usize,
}

impl Role {
    pub const fn new(id: usize) -> Self {
        Self { id }
    }

    pub const fn id(self) -> usize {
        self.id
    }
}
