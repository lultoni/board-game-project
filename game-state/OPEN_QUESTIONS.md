# OPEN QUESTIONS

*Live design questions only. Resolved/closed/scrapped/parked items: see `OPEN_QUESTIONS_ARCHIVE.md`.*

*Order: most actionable first. Each entry should have a status, a re-entry trigger (if deferred), and a short evaluation criterion.*

---

## Critical (blocks or shapes next playtest)

### OQ-21: Bodyguard Rule — COVARIATE WITH STANDOFF (P4 inconclusive)
**Hypothesis**: Bodyguard triggers more frequently with adjacent-to-defender-only rule.
- Original test artefact: previously at `docs/test-scenarios/stack-b-guards/stack-b-bodyguard-fix.typ` — *deleted Session 23*. Stack B was withdrawn Session 22 (defender-only adjacency change is no longer the proposed solution). The original rule sheet is no longer kept on disk; see the Withdrawn row in `docs/mechanics-log/mechanics-evaluated.md` for the historical record.
- **P2 update**: Bodyguard triggered 2x under baseline adjacency rule.
- **P3 update (Session 15)**: Bodyguard organically activated under Stack A nerf — standoff dissolved was the actual cause.
- **P4 update (`docs/research/playtest-4-analysis.md`)**: 0 triggers (regression). Both players: "didn't trigger / not so many move-attacks / even less kills." **Confirms Bodyguard tracks standoff state, not the rule.** When mid-game stalling returned (Armor-driven), Move-attack volume dropped, and Bodyguard had nothing to intercept. OQ-21 cannot be evaluated cleanly while standoff/stalling is a moving variable.
- **Attacker movement on intercept**: RULED — attacker moves 1 tile toward target (stops before Guard). Applied to baseline.
- **Stack B withdrawn (Session 22)**: Defender-only adjacency is unlikely to be the right fix even if Bodyguard remains broken post-Stack-H. Different solutions (e.g. simplify/remove Bodyguard per the Framing-B watch-flag below) would be on the table. The OQ itself stays open; the originally proposed stack does not.
- **Re-entry trigger**: re-evaluate Bodyguard *behaviour* after Stack H. If Armor trim reduces stalling and Move-Attacks return, Bodyguard becomes triggerable again — only then is the rule itself testable. If a fix is needed at that point, draft a new stack rather than reviving Stack B.
- **Framing-B watch-flag (added Session 20, Q-C2 finding)**: Bodyguard is the most chess-coded mechanic in the chassis — confirmed combo-irrelevant in P3 (Mario's usage was trade-up defensive screening, not combo-positioning). Not actionable today, but if a future stack proposes simplifying/removing Bodyguard for chassis-volume reasons, Framing B (ADR-004) is one of the supporting arguments. See Q-C2 in `docs/research/high-concept-open-questions.md`.

### OQ-34: Steal Balance — Mode B CONFIRMED DOMINANT (P4)
**Mode A** (opponent at 0 Money): Steal is a normal-range Strike skill (1 damage, no theft effect). Not dominant.
**Mode B** (opponent has Money): cheap damage + opponent disabling. Tempo weapon. Disable value time-dependent: early-game high impact, late-game low impact.
- **P4 evidence**: Elias Q9 *"Steal again feels very strong in some situations"* + must-pick. Niko's R26-R28 winning loop = Tempest + Steal + Focus. Niko Q15 favourite: *"stealing money to prevent skills next turn"* — celebrating Mode B. Elias R8 actively neutralised Niko's Steal champ early.
- **Decision**: hold cost increase until after Stack H. If Stack H trims Armor and combat speeds up, Steal's Mode-B value drops naturally (less time for disable to matter).
- **Re-entry trigger**: re-evaluate in next experienced-player game post-Stack H. If Mode B still dominant, raise to cost 4.

### OQ-38: Multi-Champion Combo Bonus — CONFIRMED IN MECHANICS, SCOPE WIDENING QUEUED (P4 + Session 22)
**+1 damage on the second+ Strike hit when 2+ different Champions target the same enemy this turn.**
- Buff skills targeting your own pieces do NOT count.
- Charge stacks with combo bonus (separate effects).
- Strike-only for Stack A G2 (decided Session 16). Cross-category scope reconsidered after G2 data.
- **P4 evidence (`docs/research/playtest-4-analysis.md`)**:
  - Elias R11 margin "1. ever combo!!" — first multi-Champion combo across 4 playtests (behavioural).
  - Niko's R26-R28 = 3 consecutive Strike+Strike kill rounds = winning loop.
  - 5 combo bonus activations on Elias's sheet; Niko 2/2 attempt-success.
- **Session 22 reframe (designer pushback)**: the "weak in feel" reading from the original analysis was wrong. The bonus is **by design** a few-times-a-game payoff, not an every-turn activation — a Q3 "Somewhat / Bit of both" reading is exactly right for that design intent. The lever is **scope**, not **strength**.
  - Cross-category crowd-out is a real consequence of Strike-only scope, not a bonus-strength problem.
  - Elias's late-game offensive lockout (#6) — *"i did not have any other attack champs left"* — is a scope-reachability problem too: the bonus was structurally locked away when his Strike-equipped Champs died. Single-Champion offense doesn't trigger the target counter.
- **Two design moves on the table for Stack A G3 (deferred behind Stack H)**:
  1. **Widen target counter**: any skill that hits an enemy piece counts (Strike + hit-causing Move skills like Hook). Move-Attacks excluded (free → would over-cheapen the trigger).
  2. **Add a parallel "attacker counter"**: same Champion hitting *different* targets in one turn → 2nd+ hit gets +1 damage. Encourages distributing pressure across multiple fronts (anti-exchange-pit; see new OQ on mid-game stickiness). Fixes #6 directly — single surviving offensive Champ can still access bonus by spreading hits.
  - **Stacking**: intuitive — both counters fire if both qualify on the same hit.
  - **Multi-target skills (Tempest)**: tick the counter on each hit piece. Watch flag — first surgical rollback if dual-counter proves OP.
  - **Move-Attacks**: still excluded.
- **Methodology gating**: Path A chosen — Stack H first (Armor volume), Stack A G3 (combo scope widening) after. Reasoning: Armor volume is the *confirmed* P4 problem; combo scope is solving a problem articulated structurally but not yet isolated in a single-variable test. Testing Armor first gives a cleaner baseline against which to evaluate dual-counter.
- **Teaching cost flag**: dual-counter is strictly more complex than current G2. Two parallel counters needs physical tracking solution (G4 Cognitive Load guardrail). Budget for tokens / board-side trackers in the Stack A G3 design.
- **Status**: G2 verdict held — keep into baseline. G3 (dual-counter + widened scope) staged in `backpocket.md` with full Justification Rule writeup, queued behind Stack H.

---

### OQ-58: Mid-Game Stickiness / "Exchange Pit" — OPEN (P4 finding, Session 22)
`[System: Combat / Positioning] [Affects: Mid-game pacing, parallel-puzzle texture]`
**Once a melee exchange starts, basically all the action happens there until the end of the game. Pieces get taken one-by-one in a single localised cluster; outside the cluster nothing happens.**
- **P4 evidence**: Elias D-note: *"once an exchange starts, basically all the action happens there until the end of the game."* The R15-R21 Armor cluster + R22-R28 kill loop both happened in a single board region.
- **Designer note (Session 22)**: distinct from "both players don't know what to do" — that's an early-game positioning + post-exchange endgame conversion problem. Exchange-pit is specifically the mid-game pattern *during* the cluster.
- **Distinct from OQ-11 (Armor volume)**: Even with Armor trimmed (Stack H), the *positional* convergence of all action into one pit is its own pattern. Armor stalling fills downtime; exchange-pit constrains where action happens.
- **Distinct from OQ-19 (endgame acceleration)**: First Champion kill timing is healthy (R13). The problem is not "kills come too late" — it is "all kills happen in the same place, sequentially, with no parallel pressure."
- **Hypothesised lever (Session 22)**: dual-counter combo bonus (see OQ-38 Stack A G3 plan). The proposed **attacker counter** rewards a single Champion hitting *different* targets in one turn, structurally encouraging multi-front pressure rather than one-pit convergence.
- **Other candidate levers** (no Justification Rule writeup yet — flag only):
  - Sente / threat skills (Stack F territory) that create incentive to defend a *different* part of the board.
  - Centre-attractor mechanic (overlap with OQ-52 — central tile attractors could split the action geographically).
- **Re-entry trigger**: post-Stack-H. If chassis trim doesn't reduce exchange-pit pattern, dual-counter combo (Stack A G3) is the targeted fix. If exchange-pit dissolves under Stack H alone, Q-C1 was masking it and dual-counter may not be needed.
- **Connected to**: OQ-38 (combo scope widening); OQ-52 (centre attractor); OQ-11 (chassis volume — masking variable).

### OQ-59: Opening + Endgame "Don't Know What To Do" Pattern — OPEN (P4 finding, Session 22)
`[System: Whole-game / Positioning / Strategic clarity]`
**Two distinct dead-air pockets in the same game: (a) early-game positioning (where to put pieces, no combat reachable yet); (b) post-mid-game-exchange endgame (the cluster is over, the game isn't, and neither player has a clear plan to convert the resulting position into a win).**
- **P4 evidence**: Elias D-note: *"there were a lot of turns where we both didn't really know what to do."* Designer's Session 22 clarification: this happens specifically (a) at game start before pieces meet and (b) after the central exchange ends, not generically across the whole game.
- **Distinct from OQ-58 (exchange-pit)**: exchange-pit is "all the action lives in one place during mid-game." This OQ is "before and after the exchange there is no clear strategic gradient."
- **Distinct from OQ-11 (Armor)**: Armor stalling is *what players do when they have nothing better to do.* This OQ is the underlying *why nothing better is available* — the strategic gradient is flat in those windows.
- **Sub-problem 59a — Opening incentives skew chassis-ward**: at game start, no Strike skills firing (pieces too far apart for Range 2). Only Defense + occasional Move-Attack. Move skills sit idle despite being equipped. Players move chassis-level (whole pieces forward) rather than skill-level. **Designer note (Session 22)**: *"there are no strike skills at the start ONLY defense (or sometimes move-attacks)."*
- **Sub-problem 59b — Endgame conversion gap**: when the central exchange ends, the resulting position should *suggest* a winning plan. P4: it didn't. Both players sat stumped on how to convert.
- **Candidate levers (no Justification Rule writeups yet — flag only)**:
  - For 59a: Move skill repricing in opening; OQ-1b 8×8 board (Stack K G1) reduces opening distance; combo-bonus widening to include Move skills (OQ-38) — gives Move-into-Strike offensive value early.
  - For 59b: King-as-real-target (overlaps OQ-53); cascade trigger / threat skills (Stack F); central attractor (OQ-52).
- **Re-entry trigger**: re-evaluate after Stack H + Stack A G3 + Stack K. The dead-air windows may shrink as chassis volume drops and combo grammar widens. If they persist, dedicated stack(s).
- **Connected to**: OQ-52 (centre attractor); OQ-53 (King role); OQ-1b (8×8); OQ-38 (combo scope); OQ-58 (exchange-pit — pairs with this).

### OQ-61: Two-Pole Parallel Design — Pole A vs Pole B — OPEN (Session 23)
`[System: Project methodology / Game shape]`
**Should the project carry both versions forward indefinitely as game *modes*, or is Pole B (per-turn-draft) an experiment that *replaces* Pole A (pre-game-draft) if it lands?**
- **Origin**: Session 23 discussion — see `docs/research/path-y-defense-redesign.md`. User crystallised two parallel game versions rather than continuing to tweak variables inside a single rule set.
- **Pole A** = pre-game-draft (current game). **Pole B** = per-turn-draft (skills added during play; 12-equipped cap; shared action slots; no Money activation gate).
- **Current user lean (Session 23)**: experiment-that-could-replace, with both alive while we learn. User: *"i do not wanna abandon somethign that might serve a different game feel. so like if we see that both rule versions create different feels and stuff then we should maybe think about having 2 modes for the game."*
- **Resolution criteria**: after first 2–3 Pole B prototype games, compare game-feel against Pole A. If Pole B clearly produces a different (good) feel → consider 2-modes. If Pole B feels like a clear upgrade in the same direction → Pole B replaces. If Pole B fails → Pole A track resumes.
- **Re-entry trigger**: after 2–3 Pole B prototype playtests on the digital prototype.
- **Connected to**: OQ-62 (Pole A draft information); OQ-63 (cross-pole fixing methodology); OQ-11 (Armor — diagnosed in Pole A, may persist in Pole B).

### OQ-62: Pole A Draft Information — Sequential vs Simultaneous Reveal — OPEN (Session 23)
`[System: Skill Drafting / Strategic determinism]`
**Does the current sequential pre-game draft cause a "deterministic perfect game / always better to react" pathology, and does simultaneous-reveal drafting fix it without breaking perfect-information-during-play?**
- **Origin**: Session 23 — user identified that sequential draft makes counter-picking always strictly better than committing to a strategy. *"there is no fundamental strategy picking as it is always better to react instead of doubling down."*
- **Proposal**: *"both players pick 2 skills at the same time when both are ready and repeat."* Tunable: pick-size per round (2 vs N), reveal cadence.
- **Information-loss carve-out**: User accepts limited PI loss **only** in the pre-game draft window: *"i accept losing a tiny bit of perfect information in the 'pre game part' if we uphold perfect information later on."* The in-game commitment to perfect information stands.
- **Scope**: Pole A only. Pole B has its own draft model (per-turn) and this question doesn't apply there directly.
- **Re-entry trigger**: any Pole A stack work that touches drafting; or post-Pole-B-prototype, when Pole A is re-evaluated against the new data point.
- **Connected to**: OQ-34 (Steal — affected by draft visibility); OQ-56 (draft entry complexity); OQ-16 (skill drafting fairness).

### OQ-63: Cross-Pole Fixing Methodology — OPEN (Session 23)
`[System: Project methodology]`
**When a problem (e.g. Armor's role) exists in both poles, do we run the fix in each pole separately, or once and carry across?**
- **User lean (Session 23)**: *"its cleaner if we have to run it twice, once per pole, but i also say that it could be confusng if we do not clearly seperate both poles from one another."* Two-pole-twice is the default lean, with confusion as the explicit risk.
- **Resolution criteria**: encountered first time on a real shared fix; method choice locks in then. Don't pre-decide in the abstract.
- **First likely encounter**: Armor / defense redesign (Pole-agnostic candidate in `backpocket.md`). When it triggers, the cross-pole methodology resolves.
- **Re-entry trigger**: first shared-fix proposal that applies cleanly to both poles.
- **Connected to**: OQ-61 (two-pole framing); OQ-11 (Armor — first likely shared-fix surface).

### OQ-60: Cognitive Load — Real Concern or Acceptable? — WATCH (P4 finding, Session 22)
`[System: Whole-game / G4 guardrail]`
**Both players reported the game required heavy thinking. Elias Q15 was crossed out with a "/" and the comment *"it all felt like a lot of thinking after a long day at work."* Niko's A1: "strategy not easy though."**
- **Question**: is this load *real-but-acceptable* (the depth IS the appeal — chess-deckbuilding hybrid demands thought), or is it *breaking against G4 cognitive-load guardrail* (load is not earning its keep)?
- **Designer note (Session 22)**: not framing as "is the game too long" — that's separate. Framing as "is the *amount* of thinking per turn proportional to the strategic payoff."
- **Confounders to control for in next playtest**:
  - Time-of-day / fatigue (Elias's note explicitly mentioned "after a long day at work").
  - First-game vs experienced (first-time player will always feel high load).
  - Chassis volume (OQ-11) — if Armor calculations consume mental focus, trimming Armor should reduce load without reducing depth.
- **Re-entry trigger**: post-Stack-H, ideally with two experienced + rested players. Add a feedback question: "did the *amount* of thinking per turn match the strategic payoff, or was it more thinking than the decisions justified?"
- **Connected to**: G4 in `backpocket.md`; OQ-11 (Armor volume as load source); OQ-58 (exchange-pit attention concentration).

---

## High Priority (affects system interactions)

### OQ-19: Endgame Acceleration — NOT TRIGGERED (P4)
- Checkmate-style win condition **killed** (Session 11). Research: `docs/research/checkmate-win-conditions.md`.
- **Leading candidate**: King Lifetime HP — separate irreversible damage track. See `docs/backpocket.md`.
- **Secondary candidates**: Fewer pieces (Stack D), smaller board (Stack D), threefold repetition rule (anti-stalling, simple).
- **P4 evidence (`docs/research/playtest-4-analysis.md`)**: First Champion kill **R13** (within healthy <R20 range). Game length 28-29 rounds is long, but driven by mid-game *Armor* stalling, not slow first-kill. Stack C is the wrong lever.
- **Re-entry trigger**: re-evaluate if first Champion kill ever exceeds R20. P4's R13 says Stack C remains deferred; Stack H targets the actual problem.

### OQ-26: AP System Piece Freedom — Stack 4 candidate
**How many AP can one piece receive per turn?**
Four constraint models proposed (A: 1-skill-per-piece; B: 1-AP-per-piece; C: uncapped, comeback unlock; D: max 2 AP per piece). Test all in Stack 4. Start with Model D as default.

### OQ-52: Centre of the Board Has No Attractor — OPEN
`[System: Board/Spatial] [Affects: Movement, King role, Opening dynamics]`
**The centre is mechanically inert; players naturally flank-drift at the start.**
- Three reinforcing causes: cramped opening formation, enemies clustered centre, King "stays safe" backward.
- **Designer constraint (explicit)**: NOT introducing a "queen" piece that pulls action centrally — the lever must be board-geometry, attractor mechanics, or formation, not piece power.
- **Solution space**: central rune/scoring tile; contested resource at centre; narrower board (8×10); formation rework; or accepting it.
- **Re-entry trigger**: Address before or during Stack F (Cleverness II), since sente skills assume threatenable territory.

### OQ-53: Attrition vs. Regicide — Should the King Be a Real Target? — OPEN
`[System: Win condition / Strategic texture]`
**Formal win = King capture; real victory path = attrition.**
- Playtest 3: Mario surrendered when last Champion died, not when King was threatened. Both Kings barely moved.
- **Designer's intent (Session 14)**: King should be a real threatenable target.
- **Critical clarification**: King Lifetime HP alone does NOT fix this — it makes the King durable, not participating.
- **Solution space**: starting-formation swaps (backpocket); central-attractor mechanics (overlaps OQ-52); sente threat skills targeting the King; mobility/safety asymmetry.
- **Re-entry trigger**: Brainstorm session before Stack F, OR a dedicated King-role design session.

### OQ-11: Armor Cap — CONFIRMED chassis-volume problem; STACK H QUEUED (P4 → Session 23)
`[System: Health & Armor] [Affects: Combo-loop visibility, parallel-puzzle attention]`
**Reopened from archive 2026-05-26.** Originally closed Session 15 as "RPS loop functions." High-concept audit (Angle 2) flagged Health & Armor as strongest chassis-bloat candidate. The question is now *volume*, not *balance* — does the Armor↔Armor-Breaker loop draw attention away from the combo loop both players are trying to discover (Framing B)?
- **P3 evidence**: Mario granted ~20 Armor across the game; Elias used Break ~6 times. Real chunk of game-time.
- **P4 evidence (`docs/research/playtest-4-analysis.md`)**: best evidence yet for the chassis-volume hypothesis.
  - Elias Q13: **Yes, a lot** mental focus; game **Slowed noticeably**.
  - Elias verbatim: *"armor was a part of combo calcs but it just felt like you were not able to do your combos because of it"* and *"a lot of armor was stacked also because there was nothing better to do"*.
  - Both players ran identical Armor-stack arcs: R15-R21 (Niko) / R15-R18 (Elias) = pure Armor mid-game cluster, no Atk.
  - Total Armor granted: 14 (Elias) / 22 (Niko).
  - **Session 22 designer note on the 22 vs 14 asymmetry**: Niko stacked more partly because Elias's Break champ died early before getting value, removing the chassis-side counter to runaway Armor. The Armor↔Break loop *only* functions when the Break piece survives — fragile balancing counterloop.
  - Niko's split read (Q13 "Not really" + "Slightly extended") suggests cost is **asymmetric across skill levels** — experienced player feels it more because he plans combos around it.
  - Reversal criterion check: did combos *reliably* overrun Armor? Niko's R26-R28 loop did, BUT only after a 7-round Armor consolidation. **Cannot dissolve Q-C1.**
- **Test plan (Q-C1, decided Session 20; restructured Session 22)**: Stack H bundles **C1b** as the lead dose — cap 3→2 *and* Plate +1→+2. Risky-path-first: smaller dose **C1a** (cap 3→2 only) runs as the next iteration of Stack H if the bundled dose stalls. (Previously this was tracked as a separate Stack I — collapsed into Stack H Session 22.)
- **Session 23 update — DEPRIORITISED to Queued.** Pole B per-turn-draft prototype claims the Active slot during the 3-week vacation digital-prototype window (see OQ-61). Stack H bundled dose remains the **lead variant** when Stack H runs; within-stack rollback (cap-only) is the contingency. **"Build cheaper than break" risk is bigger than originally framed** — user verbatim: *"if it is way easier to stack armor then it is to get rid of it... the change can exponetiallise this even more."* When Stack H runs, Armor totals must be tracked vs P4 baseline (14/22) and the rollback dose triggered if totals climb.
- **Status**: **QUEUED** — runs after Pole B prototype data lands (or earlier if Pole B fails fast).
- **Connected to**: Q-C1 in `docs/research/high-concept-open-questions.md`; ADR-004 (Framing B); OQ-38 (combo bonus); OQ-58 (exchange-pit / mid-game stickiness — separate problem); OQ-61 (Pole B claims active slot); OQ-63 (cross-pole fixing — Armor is the likely first shared-fix encounter).

### OQ-57: Injured State — Mechanical Downsides Carry Their Weight? — PARTIALLY CONFIRMED (P4)
`[System: Health & Armor] [Affects: Combat texture, teaching cost, combo grammar]`
**Opened 2026-05-26** from Q-B5 reframing. Original question was "should Injured be hidden for first-game teaching?" — reframed via user pushback to "should Injured have any mechanical downsides at all?" That's a stack candidate, not a teaching-protocol tweak.
- **Chassis volume claim**: Injured carries non-trivial teaching cost (speed cap, Range −1, "doesn't affect self/adjacent" carve-out, Range-modifier chaining for Range−1 skills on Injured pieces). Multiple rule clarifications during play stem from it.
- **P4 evidence (`docs/research/playtest-4-analysis.md`)**:
  - Niko C9: "at first: injury" — explicitly named Injured as a confusion source on first read (teaching-cost confirmation).
  - Niko Q12: **Clearly weaker** — felt the downsides ("also because it was close to death").
  - Elias Q12: **Slightly weaker / Barely noticeable** — experienced player barely registers the mechanical effect.
  - Split read: experienced player barely notices Injured's effect; new player gets weak-piece feel + confusion at the rule. **Suggests volume is real but mechanical payoff is thin.**
- **Test plan (Stack J)**: Remove Injured's mechanical downsides entirely. State persists as HP-tracker (2 HP → 1 HP → 0 HP) but Injured pieces have no speed cap and no Range −1.
- **Trigger / gating**: After Stack H. P4 lifts the prerequisite ("after Stack A G2") — combo bonus is now confirmed, the gating is just Stack H ordering.
- **Recognised risk**: Could scale up to baseline-change candidate if it plays well — that's accepted as part of the stack's scope, not a misframing.
- **Connected to**: Q-B5 in `docs/research/high-concept-open-questions.md`; chassis/engine lens in `design-principles.md`.

### OQ-56: Draft Entry Complexity + Skill Permanence — INCONCLUSIVE-CONTAMINATED (P4)
`[System: Skill Drafting] [Affects: New player onboarding, Mid-game adaptability]`
**Three linked problems**:
- **A — Draft entry complexity**: New players cannot evaluate skills during draft (no on-board experience). Compounded by catalogue size: more skills = more options to evaluate before the first move.
- **B — Permanent draft decisions**: Mid-game realisation that loadout doesn't fit board state, no way to adapt.
- **C — In-game reference load**: Two compounding sources.
  - *C1 — Lookup friction (UX layer)*: Constant board↔skill-list switching. Physical skill cards on the table reduce this directly.
  - *C2 — Skill identity volume (structural layer)*: Number of distinct skills a player must mentally model — own loadout + opponent's loadout. UX fixes do NOT address this. More skills equipped per side = more identities to track. Currently ~8 identities at the table (4 Champions × 2 slots × 2 players).
- **Tension with OQ-12 (catalogue expansion to ~25)**: Catalogue size primarily affects A (more options to evaluate at draft), not C2 (capped by equipped-slot count). But a larger catalogue still makes the draft harder for new players. Expansion and onboarding need to be reconciled.
- **Design constraint**: Solution must not reduce strategic depth for experienced players.
- **Solution candidates**:
  - For A: try-before-lock draft phase; simplified starter loadouts (skip draft on first game); tiered "core vs. advanced" skill catalogue (new players play core-only).
  - For B: in-game skill switching; mid-game redraft event.
  - For C1: physical skill cards (low-risk UX fix). **SHIPPED Session 18** (`shared/skill-cards.pdf`).
  - For C2: lower equipped-slot count (Stack D / OQ-27 territory); force loadout overlap (smaller draft pool, duplicates across Champions); tiered catalogue (also helps A).
- **P4 evidence (`docs/research/playtest-4-analysis.md`)**:
  - A1: "easy to understand, strategy not easy though" — rules clear, depth not surface-readable on turn 1.
  - C7 (lookup frequency): "rather often, just for the details, felt okay (1st game)" — physical skill cards helped (Q-C1 UX layer working).
  - **Q-D1 reading: contaminated.** Elias did NOT honour the teacher-vocab-checklist commitment. Wrote: *"I just used the words to make it clear what the game is about"* + pitch box noted he "explained a lot of my experience and good combos." Niko's engine vocabulary ("combinations", "skill combos", "stacking of skills") cannot be cleanly attributed to game experience — much of it could be borrowed.
  - D14: **Same puzzle** (highest Framing-B signal — but contamination flag applies).
  - D11 mixed engine + chassis vocabulary; E15: "chess, deckbuilding."
- **Verdict**: Q-D1 ≥2/4 strong-signal threshold NOT met by this session. Need ≥1 more first-timer session with proper vocab discipline before Framing B can be evaluated.
- **Process fix needed**: teacher-vocab-checklist must be more enforceable — pre-game initials per word, OR teacher reads rules from document instead of paraphrasing. Tracked in `NEXT_STEPS.md`.
- **Re-entry trigger**: Next first-timer session under improved teacher-protocol.

---

## Medium Priority (balance and polish)

### OQ-2: Money Economy Model
**Automatic confirmed.** Stack A baseline accepted. Performance-based is closed (OQ-47). Status: live but no action — kept open as the policy anchor.

### OQ-4: Skills Per Piece Per Turn
**Uncapped for now.** Charge burst is a skill-specific balance issue, not a structural one.

### OQ-6: Skill action Cost (Small vs Big)
*(Unchanged — not directly tested.)* See OQ-50 for the active proposal.

### OQ-8: Money Cap — superseded
**No cap initially.** Players naturally spend down. See OQ-46 in archive (closed from monitoring P3). Kept open only as a watch-flag.

### OQ-12: Skill Catalogue Completeness — RESEARCHED
**Real problem**: Too few distinct strategic identities within Shield (3 passive buffs) and Mystic (1 must-pick + 1 never-pick).
- **10 new skill candidates staged** in `docs/backpocket.md`.
- **Target**: ~25 skills total. Distribution: ~9 Strike / 6 Shield / 5 Move / 5 Mystic.
- **Tension with OQ-56 Problem A**: Larger catalogue = harder draft for new players. Tiered "core vs. advanced" catalogue is one way to reconcile.
- **P4 update (Session 22)**: must-pick density is **softer than originally framed**. Q10 lists looked nearly identical (Focus + Armor + Steal) but the actual draft distribution was: Focus 1-2 *total* across the army (not per Champion); Armor 2-3 *across all*. Most Champions equip neither. The "Focus + Armor on every Champ" reading was wrong — must-picks are **per-loadout**, not **per-Champion**. Catalogue density problem still exists but is less severe than P4 self-reports made it sound. Down-prioritised vs. chassis-volume work.
- **Trigger for expansion**: Stack F or dedicated session after Stack A/B confirm combat balance is stable. Do not expand mid-combat-testing.
- **Q-E1 update (Session 20)**: When this triggers, evaluate the *intervention type* — replace-for-breadth (swap an under-performing skill for a new combo shape; count flat) vs. expand-catalogue (add on top; count up). The two have very different newcomer costs. See Q-E1 in `docs/research/high-concept-open-questions.md` for the symptom-trigger framing ("experienced players report combos exhausted").
- **Connected**: New skill ideas surfaced in P4 discussion now staged in `backpocket.md` — Plague (Injured-as-payload, ignores Armor); Lucky/Star Strike (Mystic, target any opponent piece on board); Focus-as-paid-scale (replace Focus skill with "spend +1 Money for +1 Range" mechanic); Lance/Theft merge.

### OQ-22: Defensive Skill Viability
**Improved in Playtest 2.** Plate, Shield, Heal all used meaningfully. Re-evaluate after future stacks if defense feels unviable again.

### OQ-23: Move action Count
**May be superseded by AP system.** If staying with current turn structure, test 3 Move actions. If moving to AP, this question dissolves.

### OQ-24: Skill Card/Rule Clarity
**Review all skill descriptions for clarity before next playtest.** Connects to OQ-56 Problem C (skill cards on the table).

### OQ-39: Shared-Puzzle Design Direction — RESOLVED (ADR-004, 2026-05-26)
**Resolved as Framing B ("Two minds, one puzzle") — see ADR-004 in `docs/mechanics-log/mechanics-evaluated.md` and `docs/design-principles.md` § High-Concept Framing.** Design intent locked: 2-player nature is load-bearing, opponent is fellow puzzle-solver, combo legibility must work in both directions, asymmetry biased against. No immediate mechanical changes. Reversal criterion in ADR-004.
**→ Move to OPEN_QUESTIONS_ARCHIVE.md next archive pass.**

### OQ-41: Game Length vs. Damage Nerf Tradeoff — PARTIALLY CONFIRMED (P3 + P4)
**1-damage attacks did NOT noticeably extend the game in P3** — Round 24 finish.
- **P4 update (`docs/research/playtest-4-analysis.md`)**: Stack A G2 game ran 28-29 rounds (~2h30). Both players rated "a bit long" / "way too long." But the length problem is **mid-game Armor stalling**, not the 1-damage nerf — first Champion kill was R13 (faster than P2's R26). The damage nerf is fine; chassis volume is the lever.
- **Status**: Closed as "nerf does not extend; mid-game stalling is the real culprit, addressed by Stack H." Move to archive next sweep.

---

## Deferred (concrete trigger required)

### OQ-27: Piece Count and Ratio — DEFERRED
**Current: 5 Champions + 6 Guards + 1 King.** Proposed (Stack K — Piece Count Reduction): 3 Champions + 4 Guards + 1 King.
- *Decoupled from board size in Session 22.* Stack K now owns piece-count reduction only; board geometry lives in Stack D. Test independently.
- **Ratio rationale**: 4G+3C keeps a stronger bodyguard/screen function while shrinking the combo engine to 3 Champions — fewer slots to evaluate, less option-overwhelm, same chassis feel.
- **Re-entry trigger**: After Stack H — gated on chassis-volume confirmation per current TESTING_PLAN routing.
- **Test plan**: Stack K — Piece Count Reduction, single-variable. (Previously bundled with 8×8 board as a two-game session; decoupled Session 22 because piece count and board size are independent at 8×8 and 10×10.)

### OQ-1b: 8×8 Board Test — DEFERRED (residual of OQ-1)
**10×10 confirmed viable; 8×8 not yet tested. Would tighter geometry reduce empty-board feel in opening?**
- *Decoupled from OQ-27 in Session 22.* Now lives under Stack D — Board Geometry alongside 8×10 (OQ-52) and hex (OQ-42).
- **Re-entry trigger**: Stack D entry condition (board surfaces as bottleneck in any future stack).
- **Follow-up (OQ-1c, contingent)**: If 8×8 shows positive returns (denser play, less empty-board feel, shorter games), the *next* test re-bundles board + pieces at 6×6 + 3C+4G+1K. Coupling is deliberate at 6×6 (you can't fit the full piece set). Gated strictly behind positive 8×8 data + positive Stack K data.
- **History**: see OQ-1 in `OPEN_QUESTIONS_ARCHIVE.md`.

### OQ-35: Skill Pool Draft Variant — DEFERRED
**Each player drafts a pool of N skills first, then assigns skills to Champions.**
- Could reduce "accidental" synergies and increase strategic intent. Connects to OQ-56 Problem A.
- **Re-entry trigger**: Test after Stack A/B accepted. Independent of combat/economy changes.

### OQ-36: Flexible Piece Placement — DEFERRED (bundled with OQ-48)
**Players choose starting positions within constraints** rather than fixed formation.
- Risk: increases setup time and reactive counter-positioning. Explore reveal-style simultaneous pick (backpocket).
- **Re-entry trigger**: Test after Stack A/B accepted, bundled with OQ-48.

### OQ-42: Hex Grid Board — REOPENED
`[System: Board/Spatial]`
**Should we evaluate a hexagonal grid as an alternative to the square 10x10?**
- Original ADR-001 confirmed "grid over card-fighter," not "square over hex."
- **Research needed**: How do published tactical games handle hex vs. square? What does hex do to combo geometry?
- **Status**: Trigger `/research hex vs square grid in 2-player tactical board games` before any test stack is proposed.

### OQ-48: Piece Placement Order — DEFERRED (bundled with OQ-36)
**Place pieces after the skill draft, informed by loadout.**
- **Re-entry trigger**: After Stack A/B accepted, bundled with OQ-36.

### OQ-49: Path Obstruction Model — DEFERRED
**Idea 1** (only opponent pieces block) **withdrawn Session 7** (creates turtle meta).
**Idea 2** (only opponent Guards block) — not yet evaluated. Risk: Guard wall dominant strategy.
- **Re-entry trigger**: After Stack A/B — if own-piece LoS blockage is consistently frustrating, test Idea 2. Otherwise park.

### OQ-50: Minor/Major Skill action Cost — DEFERRED
**Skills cost different numbers of actions (minor=1, major=2).**
- With 2 actions/turn, 2-cost skills are traps unless designed as "ultimate" skills worth the commitment.
- **Re-entry trigger**: Design 2-3 candidate "ultimate" skills first. Only evaluate as part of Stack 4+ (actions expand to 3/turn). Connected to OQ-19 (endgame acceleration).

### OQ-51: Mechanical Levers for Rewarding Clever Plays — RESEARCHED
**Three promising candidates staged in backpocket** (Cascade trigger, Pin/Threatened, Collision damage).
- **Cascade trigger**: anti-snowball one-turn tempo bonus on kill. Test in Stack F or earlier.
- **Pin/Threatened**: restriction-as-reward. Needs own stack.
- **Collision damage**: gated on standoff resolution.
- **Status**: Researched. Next step: test cascade trigger in Stack F.

---

## Open (no clear trigger yet)

### OQ-14: Draw Conditions
*(Unchanged.)*

### OQ-3b: AP System Replaces Movement-Action Link — OPEN (residual of OQ-3)
**Unlinked Movement/Action confirmed working. The remaining question: should the AP-system stack supersede it entirely, or do they coexist?**
- Tied to OQ-26 (AP system constraint models).
- **Status**: No action until AP-system stack is on the roadmap. Decision happens when that stack is designed.
- **History**: see OQ-3 in `OPEN_QUESTIONS_ARCHIVE.md`.

### OQ-13b: First-Player Advantage — DORMANT WATCH (P4: 1 P1 win flagged unprompted)
**Note: P4 Niko (P1) won and unprompted flagged "First mover advantage?" on his first-game form. 1-of-1 is not signal — continue watch.**
- No advantage observed across P1, P2, P3.
- **Re-entry trigger**: A consistent first-player win rate across many games (5+) with similar player skill.
- **If triggered**: Apply Go-style komi (P1 starts with less Money), per OQ-45 reasoning.
- **History**: see OQ-13 in `OPEN_QUESTIONS_ARCHIVE.md`.

### OQ-20b: Default Range 2 — Practical Feel Watch (residual of OQ-20)
**Range 2 is the canonical default; question is whether it feels limiting in practice.**
- No "Range 2 too short" feedback in P2 or P3.
- **Re-entry trigger**: Player feedback in any future playtest that Range 2 skills feel too constrained.
- **History**: see OQ-20 in `OPEN_QUESTIONS_ARCHIVE.md`.

### OQ-16: Skill Drafting Fairness
- Fair but "decides a lot." Future question: is draft too deterministic? See also OQ-56.

### OQ-54: Lance — Should Its Text Say "Adjacent" Instead of "Range−1"?
**Effect text reads "Target within Range−1 takes 1 damage" — under accepted Range ruling, effective Range 1 = adjacent.**
- For rewriting: clearer at the table, no mental arithmetic.
- For keeping "Range−1": preserves the design signal that reduced range is a deliberate tradeoff.
- **Session 18 ruling**: Keep "Range−1." The derivation chain (default 2 + Range−1 = effective 1) is the rule; the modifier-form preserves the design intent. Confirmed during skill-cards build that Range−1 + Injured = effective 0 = cannot fire — a non-trivial interaction the "Adjacent" rewrite would obscure.
- **Status**: Closed for now. Re-watch only if Range−1 language causes confusion in further playtests.

### OQ-55: Charge — Broader Skill Interaction?
**Could Charge interact with a wider range of skills (e.g. boost Armor gained by Shield skills)?**
- Broader Charge deepens Mystic identity as a "combo enabler."
- Risk: universal "make your best skill better" button — possibly too flexible.
- Alternative: introduce a second Mystic skill that amplifies Shield/Move (separate amplifier archetype).
- **Status**: No action until skill catalogue expansion is unblocked. Justification required: what specific problem does broader Charge fix?
