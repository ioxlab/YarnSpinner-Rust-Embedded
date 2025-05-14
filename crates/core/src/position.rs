/// Represents a position in a multi-line string.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Position {
    /// The zero-indexed line of this position.
    pub line: usize,

    /// The zero-indexed character number of this position.
    /// Careful: This represents a unicode code point, not a byte, i.e. what you'd get with `string.chars().nth(character)`.
    pub character: usize,
}
