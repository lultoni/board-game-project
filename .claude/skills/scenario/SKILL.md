---
name: scenario
description: "Park a candidate design lever in the `backpocket` table — a problem→solution→when-to-deploy entry. Replaces the retired 'one stack per experiment' methodology; the real ruleset lives in game/ + design/RULES.md."
argument-hint: "<short description>"
---

# Candidate Lever: $ARGUMENTS

The `stacks` "one stack per experiment" methodology is **retired** (Session 45). The current ruleset lives in `game/` (the engine) and `design/RULES.md` (canonical prose). We no longer mint `stack-a…stack-z` rows for every test.

Instead, a testable change is parked as a **lever** in the `backpocket` table: a row that records **what problem it fixes** (`fixes`), **when to deploy it** (`trigger_cond`), and the full rationale (`body`). When a playtest surfaces problem X, `/start` and `/playtest` surface the parked levers so you can see "we already have candidate Y for this."

*(The 16 historical `stacks` rows are frozen for provenance — don't add to them, don't delete them. Cross-link a new lever to a frozen stack with `derived-from` if it descends from one.)*

## Step 1: Validate (Justification Rule + Methodology)

Pull the methodology + existing parked levers before designing:

```bash
sqlite3 design/design.db <<'SQL'
SELECT body FROM principles WHERE kind='methodology';
SELECT id, name, fixes, trigger_cond FROM backpocket WHERE category='staged-fix' AND status='parked' ORDER BY id;
SQL
```

Check:
1. **Justification (MANDATORY)**: what specific problem does this fix, or what game-feel improvement does it deliver? "Sounds cool" is not enough.
2. **Duplication**: does a parked lever already cover this? If so, enrich that row instead of creating a new one.
3. **Independence / attribution**: if deployed alongside other changes, can we still attribute the effect? Note coupling in the body.

Pull the current ruleset for context:

```bash
sqlite3 design/design.db "SELECT body FROM stacks WHERE id='stack-n';"   # latest change set (for reference)
```
Also read `design/RULES.md` (canonical current rules).

## Step 2: Pick an ID

Convention: `bp-<kebab-slug>` describing the lever (e.g. `bp-forward-guard-partial-skill-immunity`). No letters, no sequence — the slug is the identity.

## Step 3: Write the lever body

The `body` is full markdown. Suggested sections:

```markdown
# <Name> — lever

## Mechanic
[What the change actually is — precise enough to implement.]

## What problem it fixes  (→ the `fixes` column, condensed)
[The specific problem / OQ this targets. Cite playtest evidence.]

## When to deploy  (→ the `trigger_cond` column, condensed)
[The condition under which this lever should be pulled — e.g. "if the oq-58 standoff persists after Stack N."]

## Justification (MDA)
[Mechanic → Dynamic → Aesthetic. Why it fixes the problem, and the risk it carries.]

## Routing / interactions
[How it interacts with other levers; what to watch if deployed.]
```

## Step 4: Insert into DB

```bash
sqlite3 design/design.db <<SQL
BEGIN;

INSERT INTO backpocket (id, name, category, status, fixes, trigger_cond, body, created_in) VALUES (
  'bp-<slug>',
  '<Name>',
  'staged-fix',
  'parked',
  '<one-line: the problem / OQ this fixes>',
  '<one-line: when to deploy this lever>',
  '<full markdown body from Step 3>',
  'session-<N>'
);

-- Link to the OQ(s) this lever addresses
INSERT INTO links (from_id, to_id, relation, note) VALUES
  ('bp-<slug>', 'oq-N', 'addresses', NULL);

-- If it descends from a frozen historical stack, trace it
INSERT INTO links (from_id, to_id, relation, note) VALUES
  ('bp-<slug>', 'stack-<X>', 'derived-from', 'Lever descends from retired stack row.');

COMMIT;
SQL
```

*(`created_in` is a nullable FK to `sessions` — set it to the current `session-<N>` once that row exists (created at `/wrapup`), or leave NULL and let wrapup stamp it.)*

## Step 5: Update affected rows

- `next_steps`: if the lever needs engine work to become testable, insert a `next_steps` row ("implement <lever> in game/") owned by the OQ.
- `mechanics`: if the lever stages a genuinely new mechanic, insert into `mechanics` with `verdict='staged'` and a `link` (`evidence-for`).

## Step 6: Confirm

Output a short summary:
1. Lever ID + name + status (`parked`).
2. The problem it fixes (one sentence) and its deploy trigger.
3. OQs addressed.
4. Any methodology concern (coupling, duplication, attribution).
