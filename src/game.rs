use std::collections::HashSet;
use uuid::Uuid;

/// Represents a currently active game.
pub struct ActiveGame {
    /// All the players who are currently in this game.
    players: HashSet<Uuid>,
}
