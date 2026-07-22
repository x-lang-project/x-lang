use crate::Place;

impl Place {
    /// Dereferences end the owner chain; dependent references carry their owners.
    pub fn invariant_owner_chain(&self) -> Vec<&Self> {
        let mut owners = vec![self];
        let mut projection = self;

        while let Self::Field { base, .. } | Self::Index { base, .. } = projection {
            owners.push(base);
            projection = base;
        }

        owners
    }
}
