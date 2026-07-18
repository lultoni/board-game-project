// Inspector store - tree-of-positions model for the /inspector/ route.
//
// Each node is identified by `id` and carries the cumulative `actions[]` from
// `startFen`. Branching = appending a child whose `actions = [...parent.actions, edge]`.
// Selection (`currentId`) drives what the board renders; engine state is
// resynced by replaying the selected node's snapshot.

import type { PositionView } from "../engine";

export interface InspectorNode {
  id: string;
  parent: string | null;
  /** u32 Action ids, cumulative from root. */
  actions: number[];
  /** The single action that produced this node from its parent. null at root. */
  edgeAction: number | null;
  /** Cached FEN for quick display / preview. May be empty until first visit. */
  fen: string;
  /** "Point of interest" label, or null. */
  label: string | null;
  /** Depth in the tree (== actions.length). */
  ply: number;
  children: string[];
}

export interface InspectorTree {
  startFen: string;
  /** Engine Config JSON captured when the tree was built. */
  configJson: string;
  rootId: string;
  nodes: Record<string, InspectorNode>;
  currentId: string;
  nextId: number;
}

interface InspectorState {
  tree: InspectorTree | null;
  /** Live position for the currently-selected node - set by the route after
   *  the engine resyncs. Drives the board render. */
  position: PositionView | null;
  /** Latest legal actions for the current node, for the action picker. */
  legal: Uint32Array;
  /** Last "Ask AI" result, kept around so the banner can persist. */
  lastAiHint: AiHint | null;
}

export interface AiHint {
  /** raw u32 action; 0 if no move. */
  best: number;
  score: number;
  depth: number;
  /** Node id this hint was computed for; cleared when the user navigates away. */
  forNodeId: string;
}

export const inspector = $state<InspectorState>({
  tree: null,
  position: null,
  legal: new Uint32Array(),
  lastAiHint: null,
});

function freshId(t: InspectorTree): string {
  const id = `n_${t.nextId}`;
  t.nextId += 1;
  return id;
}

/** Build a fresh tree with a single root node. */
export function initTree(opts: {
  startFen: string;
  configJson: string;
  rootFen: string;
}): InspectorTree {
  const root: InspectorNode = {
    id: "n_0",
    parent: null,
    actions: [],
    edgeAction: null,
    fen: opts.rootFen,
    label: "[start]",
    ply: 0,
    children: [],
  };
  return {
    startFen: opts.startFen,
    configJson: opts.configJson,
    rootId: root.id,
    nodes: { [root.id]: root },
    currentId: root.id,
    nextId: 1,
  };
}

/** Append a child to `tree.nodes[parentId]`. Returns the new node id. */
export function addChild(
  tree: InspectorTree,
  parentId: string,
  edgeAction: number,
  childFen: string,
): string {
  const parent = tree.nodes[parentId];
  if (!parent) throw new Error(`inspector: unknown parent ${parentId}`);
  const id = freshId(tree);
  const child: InspectorNode = {
    id,
    parent: parentId,
    actions: [...parent.actions, edgeAction >>> 0],
    edgeAction: edgeAction >>> 0,
    fen: childFen,
    label: null,
    ply: parent.ply + 1,
    children: [],
  };
  tree.nodes[id] = child;
  parent.children.push(id);
  return id;
}

/** If `parent` already has a child whose edge equals `action`, return it. */
export function findChildByEdge(
  tree: InspectorTree,
  parentId: string,
  action: number,
): string | null {
  const parent = tree.nodes[parentId];
  if (!parent) return null;
  const a = action >>> 0;
  for (const cid of parent.children) {
    if (tree.nodes[cid].edgeAction === a) return cid;
  }
  return null;
}

export function selectNode(tree: InspectorTree, nodeId: string): void {
  if (!tree.nodes[nodeId]) return;
  tree.currentId = nodeId;
}

export function markPoi(tree: InspectorTree, nodeId: string, label: string): void {
  const n = tree.nodes[nodeId];
  if (!n) return;
  n.label = label.trim() || null;
}

export function unmarkPoi(tree: InspectorTree, nodeId: string): void {
  // Root keeps its synthetic "[start]" label.
  const n = tree.nodes[nodeId];
  if (!n) return;
  n.label = nodeId === tree.rootId ? "[start]" : null;
}

export function poiNodes(tree: InspectorTree): InspectorNode[] {
  const out: InspectorNode[] = [];
  for (const id in tree.nodes) {
    const n = tree.nodes[id];
    if (n.label && id !== tree.rootId) out.push(n);
  }
  out.sort((a, b) => a.ply - b.ply || a.id.localeCompare(b.id));
  return out;
}

/** Walk children in DFS pre-order. Used by the tree panel renderer. */
export function dfs(tree: InspectorTree): InspectorNode[] {
  const out: InspectorNode[] = [];
  const stack: string[] = [tree.rootId];
  while (stack.length) {
    const id = stack.pop()!;
    const n = tree.nodes[id];
    out.push(n);
    // Push children in reverse so leftmost is visited first.
    for (let i = n.children.length - 1; i >= 0; i--) stack.push(n.children[i]);
  }
  return out;
}

/** Build a snapshot JSON the engine can restore for `node`. If the tree
 * was built without a `configJson` (e.g. a stale pasted tree from before
 * the field was required), the caller can supply `fallbackConfigJson`
 * which is also written back onto the tree to repair it. */
export function buildSnapshotForNode(
  tree: InspectorTree,
  node: InspectorNode,
  fallbackConfigJson?: string,
): string {
  // Mirror `core_engine::session::Snapshot` shape: { start_fen, actions, config }.
  // We piggyback off the configJson captured at tree-build time.
  if (!tree || typeof tree.configJson !== "string") {
    if (typeof fallbackConfigJson !== "string") {
      throw new Error("inspector: tree has no configJson - was it loaded correctly?");
    }
    tree.configJson = fallbackConfigJson;
  }
  const cfg = JSON.parse(tree.configJson);
  return JSON.stringify({
    start_fen: tree.startFen,
    actions: node.actions,
    config: cfg,
  });
}

export function serializeTree(tree: InspectorTree): string {
  return JSON.stringify(
    {
      startFen: tree.startFen,
      configJson: tree.configJson,
      rootId: tree.rootId,
      currentId: tree.currentId,
      nextId: tree.nextId,
      nodes: tree.nodes,
    },
    null,
    2,
  );
}

export function loadTree(json: string): InspectorTree {
  const parsed = JSON.parse(json) as InspectorTree;
  // Minimal shape check.
  if (
    !parsed ||
    typeof parsed.startFen !== "string" ||
    typeof parsed.configJson !== "string" ||
    typeof parsed.rootId !== "string" ||
    typeof parsed.nodes !== "object"
  ) {
    throw new Error("inspector: invalid tree JSON");
  }
  return parsed;
}

export function resetInspector(): void {
  inspector.tree = null;
  inspector.position = null;
  inspector.legal = new Uint32Array();
  inspector.lastAiHint = null;
}
