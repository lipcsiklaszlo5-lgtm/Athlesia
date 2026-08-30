use std::collections::HashMap;

use crate::{Role, StructuralSequence};

#[derive(Clone, Debug, Default)]
pub struct Encoder;

impl Encoder {
    pub const fn new() -> Self {
        Self
    }

    pub fn encode<T>(&self, values: &[T]) -> StructuralSequence
    where
        T: Eq + std::hash::Hash,
    {
        let mut identities: HashMap<&T, usize> = HashMap::new();
        let mut next_id = 0usize;
        let mut roles = Vec::with_capacity(values.len());

        for value in values {
            let role_id = match identities.get(value) {
                Some(existing) => *existing,
                None => {
                    let assigned = next_id;
                    identities.insert(value, assigned);
                    next_id += 1;
                    assigned
                }
            };

            roles.push(Role::new(role_id));
        }

        StructuralSequence::new(roles)
    }
}
