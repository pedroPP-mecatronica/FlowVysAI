//! LBM streaming step — STUB. Implementation in Sprint S3.

/// Streaming boundary handling strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamingScheme {
    /// Standard pull scheme: f_new(x) = f_old(x - e_i)
    Pull,
    /// Standard push scheme: f_new(x + e_i) = f_old(x)
    Push,
    /// Esotwist (swap) scheme for memory efficiency.
    EsoTwist,
}

impl Default for StreamingScheme {
    fn default() -> Self {
        Self::Pull
    }
}
