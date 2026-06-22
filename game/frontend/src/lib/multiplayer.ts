// Multiplayer placeholder — PeerJS-based P2P with commit-reveal lockstep.
// See ADR-005 Layer 7 for the protocol.

// TODO:
//   - connect(sessionId): host or join
//   - sendCommit(actionHash)
//   - sendReveal(action)
//   - on incoming action: verify hash, pass to engine.applyAction
//   - exchange Zobrist hashes at turn boundary; full-resync on mismatch
