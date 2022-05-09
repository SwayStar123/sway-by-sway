/// The purity of a function is related to its access of contract storage. If a function accesses
/// or could potentially access contract storage, it is [Purity::Impure]. If a function does not utilize any
/// any accesses (reads _or_ writes) of storage, then it is [Purity::Pure].
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Purity {
    Pure,
    Reads,
    Writes,
    ReadsWrites,
}

impl Purity {
    pub fn can_call(&self, other: Purity) -> bool {
        match self {
            Purity::Pure => other == Purity::Pure,
            Purity::Reads => other == Purity::Pure || other == Purity::Reads,
            Purity::Writes => other == Purity::Pure || other == Purity::Writes,
            Purity::ReadsWrites => true,
        }
    }
}

impl Default for Purity {
    fn default() -> Self {
        Purity::Pure
    }
}

/// Utility to find the union of purities.  To 'promote' Reads to Writes we want ReadsWrites, and
/// the same for Writes to Reads.
pub fn promote_purity(from: Purity, to: Purity) -> Purity {
    match (from, to) {
        (Purity::Reads, Purity::Writes)
        | (Purity::Writes, Purity::Reads)
        | (Purity::ReadsWrites, _) => Purity::ReadsWrites,
        _otherwise => to,
    }
}
