use crate::Role;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuralSequence {
    roles: Vec<Role>,
}

impl StructuralSequence {
    pub fn new(roles: Vec<Role>) -> Self {
        Self { roles }
    }

    pub fn len(&self) -> usize {
        self.roles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.roles.is_empty()
    }

    pub fn roles(&self) -> &[Role] {
        &self.roles
    }

    pub fn role_at(&self, position: usize) -> Option<Role> {
        self.roles.get(position).copied()
    }
}
