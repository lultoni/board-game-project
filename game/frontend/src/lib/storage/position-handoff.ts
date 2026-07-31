// In-memory handoff of a position FEN from replay/inspector → position-builder.
let _pending: string | null = null;
export function setPendingPositionFen(fen: string): void { _pending = fen; }
export function consumePendingPositionFen(): string | null {
  const v = _pending;
  _pending = null;
  return v;
}
