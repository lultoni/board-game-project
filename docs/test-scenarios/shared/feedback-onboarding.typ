// feedback-onboarding.typ — first-game onboarding feedback form
// Use for any player playing the game for the FIRST time.
// Independent of stack — captures onboarding experience, not stack-specific data.
// Fill out IMMEDIATELY after the game, before the standard stack feedback form.
//
// LAYOUT NOTE: Designed for exactly 2 pages (printed double-sided = 1 sheet).
// Page 1 = sections A/B/C (8 questions). Page 2 = sections D/E (7 questions).
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

#fq("4")[*How did you decide which skills to pick during the draft? Walk through your thinking.*]
#v(1fr)
#fq("5")[*Were there skills you didn't really understand until you saw them used in the game?*]
#v(1fr)
#fq("6")[*If you played again tomorrow, would you draft differently? Why?*]
#v(1fr)

== C — Playing the Game

#fq("7")[*During the game, how often did you need to look at your skill cards or the rules to remember what something does? Describe how that felt.*]
#v(1fr)
#fq("8")[*What about your opponent's skills — did you have a sense of what they could do, or did each move feel surprising?*]

#pagebreak()

#fq("9")[*Were there moments where you knew what you wanted to do but couldn't figure out how to do it with your skills?*]
#v(1fr)
#fq("10")[*Were there moments where you felt completely lost — like you had no idea what to do?*]
#v(1fr)

== D — How the Game Felt

#fq("11")[*In your own words, what is this game about? What is the player trying to do?*]
#v(1fr)
#fq("12")[*What was the most fun moment for you?*]
#v(1fr)
#fq("13")[*What was the most frustrating moment for you?*]
#v(1fr)

== E — Anchoring

#fq("14")[*What other game does this feel closest to, if any? It doesn't have to be a board game — video games, card games, anything counts.*]
#v(1fr)
#fq("15")[*Anything else you want to say about the experience — anything we haven't asked about?*]
#v(1fr)
