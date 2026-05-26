// feedback-onboarding.typ — first-game onboarding feedback form
// Use for any player playing the game for the FIRST time.
// Independent of stack — captures onboarding experience, not stack-specific data.
// Fill out IMMEDIATELY after the game, before the standard stack feedback form.
//
// FACILITATOR NOTE: File the teacher-vocab-checklist alongside this form BEFORE
// reading Q11. Q11 ("what is this game about?") is the primary Q-D1 signal — it is
// only interpretable if the checklist records which combo-coded words were used
// during teaching. Staple the checklist to this form before filing.
//
// LAYOUT NOTE: Designed for exactly 2 pages (printed double-sided = 1 sheet).
// Page 1 = sections A/B (6 questions). Page 2 = sections C/D/E (10 questions).
// Each page uses #v(1fr) between questions so writing space distributes evenly.
// Do NOT add fixed `#v(Ncm)` spacers — they break the auto-distribution.

#import "./template.typ": *
#show: template.with(title: "First-Game Feedback")

= First-Game Feedback

_For players experiencing this game for the first time. Fill this out before the standard feedback form, while the experience is fresh. Write on the back if you run out of room._

#grid(
  columns: (1fr, 1fr),
  gutter: 8pt,
  [*Date:* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
  [*Game length (rounds):* \_\_\_\_\_],
  [*Your name:* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
  [*Opponent:* \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_],
)

== A — Learning the Rules

#fq("1")[*How did the rules feel when they were first explained to you?*]
#v(1fr)
#fq("2")[*Was there a rule you had to ask about again during the game? Which one, and what was unclear?*]
#v(1fr)
#fq("3")[*At what point in the game did you start feeling like you understood what you were doing?*]
#v(1fr)

== B — The Skill Draft

#fq("4")[*How did you decide which skills to pick during the draft? Walk through your thinking.*\
_Did you think about how skills might work together — or did you pick each skill for what it does on its own?_]
#v(1fr)
#fq("5")[*Were there skills you didn't really understand until you saw them used in the game?*]
#v(1fr)
#fq("6")[*Was there a moment in the game where you suddenly saw how two of your skills could work together — something you hadn't planned at the start? Describe what happened.*]
#v(1fr)

#pagebreak()

== C — Playing the Game

#fq("7")[*During the game, how often did you need to look at your skill cards or the rules to remember what something does? Describe how that felt.*]
#v(1fr)
#fq("8")[*What about your opponent's skills — could you follow what they were doing, or did each move feel surprising?*]
#v(1fr)
#fq("9")[*Were there moments where you were confused by the rules, pieces, or how the board worked — things like movement, attacks, or Armor? Describe.*]
#v(1fr)
#fq("10")[*Were there moments where you had a skill plan in mind but it didn't come together — either because you couldn't see how to set it up, or it failed when you tried? Describe.*]
#v(1fr)

== D — How the Game Felt

#fq("11")[*In your own words, what is this game about? What is the player trying to do?*]
#v(1fr)
#fq("12")[*What was the most fun moment for you? Describe what happened — what led up to it and why it felt good.*]
#v(1fr)
#fq("13")[*What was the most frustrating moment for you?*]
#v(1fr)
#fq("14")[*At any point did you feel like you and your opponent were both solving a similar kind of puzzle — figuring out the same game from opposite sides? Or did it feel more like a direct "stop them from winning" contest?*\
Same puzzle — Bit of both — Direct contest — Hard to say]
#v(1fr)

== E — Anchoring

#fq("15")[*What other game does this feel closest to, if any? It doesn't have to be a board game — video games, card games, anything counts.*]
#v(1fr)
#fq("16")[*Anything else you want to say about the experience — anything we haven't asked about?*]
#v(1fr)

_Facilitator: staple the teacher-vocab-checklist to this form before filing._
