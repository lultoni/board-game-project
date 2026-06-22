//! Magic bitboards — precomputed straight-line ray attacks for each square,
//! indexed by occupancy. Built once at engine init, then O(1) per query.
//!
//! For Stack M's 8×8 board this is the classic chess approach: rook-style
//! rays for cardinal directions and bishop-style for diagonals, OR'd together
//! as the queen-style range used by most skill targeting.

// TODO: rook/bishop magic number tables.
// TODO: occupancy → attack lookup function.
// TODO: lazy_static or OnceLock init.
