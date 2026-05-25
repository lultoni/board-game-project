// teacher-vocab-checklist.typ — pre-game vocabulary checklist for the rule-explainer
// Use ALONGSIDE feedback-onboarding for any first-time player session.
//
// PURPOSE: Q-D1 (does the high concept land for new players?) is biased if the
// teacher seeds combo-first vocabulary during rule explanation. This form
// captures what the teacher actually *said*, so it can be compared against the
// player's Q11 ("in your own words, what is this game about?") afterwards.
//
// WORKFLOW:
//   1. Print this form. Fill out the TOP section IMMEDIATELY before teaching
//      (commit to which words you'll avoid).
//   2. Teach the game.
//   3. Fill out the BOTTOM section IMMEDIATELY after teaching ends, BEFORE
//      the game starts. Mark every word you actually used.
//   4. After the game, file this with the player's onboarding form.
//
// ANALYSIS LOGIC:
//   - Player said "combo" + teacher did NOT say "combo" → strong signal (emerged).
//   - Player said "combo" + teacher said "combo" repeatedly → weak signal (taught).
//   - Player did NOT say combo-first vocab + teacher avoided it → real result (didn't land).

#import "./template.typ": *
#show: template.with(title: "Teacher Vocab Checklist")

= Teacher Vocab Checklist (Pre-Game)

_For the rule-explainer to fill out around teaching. Goal: separate vocabulary the player *generated from the experience* from vocabulary the player *received from the teacher*. File alongside the player's onboarding feedback form._

#grid(
  columns: (1fr, 1fr),
  gutter: 8pt,
  [*Date:* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
  [*Player taught:* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
  [*Teacher (you):* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
  [*Stack / rule state:* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
)

== Before teaching — commit to avoidance

_Cross out the words you will deliberately avoid using during this rule explanation. The point is not to teach the game wrong — it's to not seed the vocabulary the player would otherwise have to invent themselves._

#table(
  columns: (1fr, 1fr, 1fr),
  table.header([Combo-coded], [Combo-coded], [Combo-coded]),
  [combo], [setup], [chain],
  [sequence], [synergy], [build],
  [amplify], [enabler], [payoff],
)

_Note: chassis-coded words (move, capture, position, control, attack, defend, kill) are part of the rules and should NOT be avoided — they are necessary for explanation. We are tracking which combo-coded words leak into teaching, not which chassis words do._

#v(0.5cm)

*Combo-coded words I will deliberately avoid this session:*

#v(0.8cm)
\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_

#v(0.8cm)
\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_

#v(0.5cm)

*One-sentence pitch I will use to introduce the game (write it out exactly):*

#v(1cm)
\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_

#v(1cm)
\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_

#pagebreak()

== After teaching — record what actually happened

_Fill out IMMEDIATELY after teaching ends, before the game starts. Be honest — this is for analysis, not self-evaluation._

*Of the combo-coded words above, which ones did you actually use during teaching?* (mark each one)

#table(
  columns: (auto, 1fr, auto),
  table.header([Word], [Used? (✗ once / ✗✗ multiple / — not used)], [Context (which rule were you explaining?)]),
  [combo],     [], [],
  [setup],     [], [],
  [chain],     [], [],
  [sequence],  [], [],
  [synergy],   [], [],
  [build],     [], [],
  [amplify],   [], [],
  [enabler],   [], [],
  [payoff],    [], [],
)

#v(0.5cm)

*Other combo-flavoured language you used that isn't on the list above:*

#v(1cm)
\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_

#v(1cm)
\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_

#v(0.5cm)

*Did the player use any combo-coded vocabulary during the teaching itself* (e.g., asking "so this is like a combo?" or "is this how I'd build a sequence?")? If so, which words and in what context?

#v(1cm)
\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_

#v(1cm)
\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_

#v(0.5cm)

*Anything else about the teaching session that might affect how the player describes the game afterwards?* (e.g., they had played a similar game before, you accidentally demoed a combo, etc.)

#v(1.2cm)
\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_

#v(1.2cm)
\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
