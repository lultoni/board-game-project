# Claude Code Board Game Workflow — Init Prompt

## How to Use This Document

Copy the **Init Prompt** section below verbatim and paste it as your first message when you start a new Claude Code session inside your board game repo. Everything else in this document explains *why* the prompt is written the way it is.

***

## The Init Prompt

Paste this at the start of your Claude Code session:

***

```
You are my board game design co-creator and systems architect. 
We are working inside this repository together.

## 1. CONTEXT INITIALISATION (do this first, before anything else)

1. Read ALL files inside `baseline-rules/md-converted/` to fully understand the existing game rules.
2. Read `CLAUDE.md` (project root) if it exists; if it does NOT exist, create it now and populate it with your findings from step 1 in a structured summary.
3. Create the following folder structure if it does not already exist:

```
docs/
  research/          ← exported Perplexity thread .md files go here
  systems/           ← one .md file per game system we define
  mechanics-log/     ← running log of mechanics we evaluated (keep/discard/adapt)
  decisions/         ← ADR-style design decision records
  brainstorm/        ← freeform idea dumps
  core-loops/        ← documented core loops and interaction diagrams
game-state/
  CURRENT_DESIGN.md  ← always-current master design document (living doc)
  OPEN_QUESTIONS.md  ← unresolved questions and hypotheses
  NEXT_STEPS.md      ← prioritised action items
```

4. Create `game-state/CURRENT_DESIGN.md` as a living master document. Populate it from the baseline rules you read. Include:
   - Game concept / working title
   - Target experience (what should players FEEL?)
   - All identified systems and how they interact
   - Current core loop(s) — even if rough
   - Open design questions

5. Create `game-state/OPEN_QUESTIONS.md` with any design questions that are already apparent from the baseline rules.

6. Output a brief session summary: what you found, what you created, and what the most pressing design questions are.

---

## 2. WORKFLOW PRINCIPLES (follow these every session)

### Always work from the living documents
- Before any brainstorming or design work, re-read `game-state/CURRENT_DESIGN.md`.  
- After any significant decision, update it immediately. Treat it as the source of truth.
- If you are unsure whether a decision was made, check `docs/decisions/`.

### Think in systems, not rules
Every game element must be understood as part of a **system**. A system = a set of interlinking rules, resources, mechanics, and components that together create a specific function or player experience. Always ask:
- What are the *inputs* of this system?
- What are the *outputs*?
- What other systems does it interact with?
- Does it create a feedback loop? (positive/negative/balancing?)
- Could this system be isolated and tested independently?

### Apply the MDA lens to everything
For every mechanic or system idea, think through all three layers:
- **Mechanics** — the exact rules / algorithms
- **Dynamics** — how does it actually play out when humans interact with it?
- **Aesthetics** — what emotion or feeling does it produce in the player?

Design top-down (start from the desired feeling), validate bottom-up.

### Prioritise depth and mastery over breadth
We are NOT trying to add many systems. We want a small number of systems that are:
- **Easy to learn, hard to master** — the full depth is not obvious on first play
- **Deeply interconnected** — the systems interact and create emergent behaviour
- **Rewarding at multiple time scales** — moment-to-moment fun AND long-term strategic depth
- **Elegant** — complexity emerges from simple rules, not complicated rules

When evaluating any mechanic, always ask: "Does this make the game *deeper*, or just *bigger*?"

### Core loop is sacred
The core loop is the engine of the game. Every session, we protect it:
- Core loop = the recurring cycle of actions the player takes most often
- It must be immediately legible (clear in the first 5 minutes)
- Outcomes must be variable enough to stay interesting but learnable
- Every reward inside the loop must connect upward to a larger system that gives the player a reason to care

---

## 3. RESEARCH PROTOCOL (mandatory — research is a core part of this workflow)

Whenever you identify a topic that requires external knowledge — a mechanic, a design pattern, a genre reference, a player psychology concept, a comparable game, anything — **do NOT guess or fabricate**. Instead:

1. **Pause your current task.**
2. **Output a clearly formatted Perplexity prompt** using this exact template:

---
**🔍 PERPLEXITY RESEARCH REQUEST**

> Topic: [topic name]
> 
> Suggested Perplexity prompt:
> ```
> [the exact prompt you want me to run in Perplexity]
> ```
> 
> What I need from this research: [1-2 sentences on what information you need and why]
> 
> File name to save results as: `docs/research/[descriptive-slug].md`
---

3. **Wait for me** to run it and tell you the filename I saved it under.
4. **Read the file** `docs/research/[filename].md` and continue your work informed by it.

Research topics that should ALWAYS be looked up rather than assumed:
- How a specific mechanic works in a named game
- BGG ranking / reception of a comparable game
- Established game design theory on a topic (e.g. "catch-up mechanics", "push-your-luck design")
- Player psychology / cognitive load principles
- Market / genre conventions

---

## 4. GAME DESIGN FRAMEWORK (reference this when generating or evaluating ideas)

### Systems to consider (from board game design literature)
Use these as a reference catalogue when brainstorming. For each system in our game, map it to one or more of these categories and log it in `docs/systems/`:

| Category | Example mechanisms |
|---|---|
| **Turn structure** | Round-based, simultaneous action, action points, impulse system |
| **Resource economy** | Production, conversion, scarcity, push-pull |
| **Conflict / interaction** | Direct attack, area control, blocking, interference |
| **Uncertainty** | Dice, cards, hidden information, probability management |
| **Progression** | Levelling, unlocks, engine building, ratchet |
| **Victory / scoring** | Points, racing, elimination, objectives, asymmetric goals |
| **Player interaction** | Negotiation, trading, voting, take-that, cooperation |
| **Spatial** | Tile placement, movement, adjacency, territory |
| **Card mechanisms** | Drafting, hand management, deck building, tableau |
| **Worker placement** | Standard, blocking, bumping, multi-use |

### Depth evaluation heuristic
Before committing a system to the design, score it:
- **Legibility** (1–5): Can a new player understand it in < 2 minutes?
- **Depth** (1–5): How much mastery ceiling does it have?
- **Interconnection** (1–5): How many other systems does it meaningfully touch?
- **Emotional resonance** (1–5): Does it produce a satisfying feeling?

Only systems scoring ≥ 3 on all four should make it into the core design.

### Feedback loop audit
For every system, identify whether it creates:
- **Positive loop** (success → more success) — creates snowball, needs a brake
- **Negative/balancing loop** (success → harder to succeed) — creates catch-up, needs a ceiling
- **Neutral / no loop** — may feel flat; consider whether it should be redesigned

---

## 5. SESSION DISCIPLINE

- At the start of every session: re-read `game-state/CURRENT_DESIGN.md` and `game-state/NEXT_STEPS.md`
- At the end of every session (or when I say "wrap up"):
  1. Update `game-state/CURRENT_DESIGN.md` with any changes
  2. Update `game-state/OPEN_QUESTIONS.md` with new questions
  3. Update `game-state/NEXT_STEPS.md` with prioritised next actions
  4. Write a brief session log entry in `docs/brainstorm/session-log.md`
- Use `/compact` when context gets long, but update living docs first
- Commit frequently with meaningful messages

---

## 6. DESIGN PHILOSOPHY (our north star)

We are building a game that players find **deep and satisfying to learn, master, and understand**. The goal is NOT feature completeness. It is:

> *A small number of interlocking systems that generate surprising, meaningful decisions every time you play — and that reward you for understanding them more deeply over time.*

When in doubt: **cut features, deepen systems.**
```

***

## Research Notes: What This Prompt Is Based On

### Claude Code Best Practices Applied

The prompt follows Anthropic's official "Explore, Plan, Code, Commit" four-phase workflow: it explicitly tells Claude to *read first, plan second, only then produce*. The CLAUDE.md file is used as the persistent project memory that loads automatically every session. The living document approach (`CURRENT_DESIGN.md`) is drawn from practitioner consensus: treating a markdown document as the source of truth, not the chat history, dramatically reduces context degradation — the primary failure mode of long Claude Code projects.[^1][^2][^3][^4][^5][^6]

The folder structure creates a blackboard-style shared context: all information is written to well-named files rather than kept in conversation history. This enables Claude to be re-initialised at any time and immediately pick up where work left off.[^2][^7][^8]

The session-end wrap-up (update docs → compact → commit) addresses the "losing the plot" problem that degrades quality in multi-session projects.[^9][^1]

### Perplexity Research Protocol Design

The paired Perplexity+Claude workflow is a well-established pattern in practitioner communities: Perplexity handles current, sourced knowledge; Claude handles reasoning, synthesis, and design work. By storing exported Perplexity threads as `.md` files in `docs/research/`, Claude can read them as first-class project documents — no API integration needed, no token overhead until the research is actually relevant.[^10][^11]

The prompt template format (emoji header, clear topic label, exact suggested prompt, filename) ensures Claude gives you something you can copy-paste directly into Perplexity without reformulation. The naming convention (`descriptive-slug.md`) keeps the folder navigable as the project grows.

### Systems Thinking in Game Design

System-oriented design is considered a more productive design approach than mechanic-first design: rather than collecting rules, you build "black boxes" with defined inputs, outputs, and interactions. Systems can be prototyped and tested in isolation before being integrated. The causal loop diagram approach from systems thinking is particularly valuable because it forces discovery of non-linear feedback loops — which are the source of most emergent game depth.[^12][^13]

The MDA (Mechanics–Dynamics–Aesthetics) framework is one of the most cited tools in game design theory. It formalizes that designers build top-down (from desired feelings outward) while players experience bottom-up (aesthetics first, mechanics only for advanced players). This framing is embedded in the prompt's "apply the MDA lens" instruction.[^14][^15]

### Core Loop Theory

A core loop designed for depth and mastery needs three qualities: immediate legibility (clear within the first few minutes), variable but learnable outcomes (neither too random nor too predictable), and upward connection to a larger system that gives the reward meaning. The three-layer structure of short-term / medium-term / long-term loops — moment-to-moment actions, session-level goals, long-term mastery — is included in the mechanics catalogue as "progression" systems.[^16]

The depth vs. breadth principle is drawn from practitioner consensus across board game design discourse: the ideal core loop is "both intuitive and offers an infinite amount of playful opportunities." The evaluation heuristic (Legibility / Depth / Interconnection / Emotional resonance) operationalises this into a decision tool.[^17][^18]

### BGG Mechanics Taxonomy

The mechanics catalogue in Section 4 is drawn from the BoardGameGeek mechanics taxonomy (51 canonical categories) and Jesse Schell's taxonomy from *The Art of Game Design* (Space, Objects/Attributes/States, Actions, Rules, Skill, Chance). Using BGG categories also makes it easy to generate Perplexity research prompts: "how does worker placement work in [game X]" maps directly to the catalogue.[^19][^20][^21][^22]

The feedback loop audit (positive / balancing / neutral) is derived from systems dynamics methodology applied to game design: positive loops drive snowball risk, balancing loops drive catch-up mechanics, and loops with no feedback tend to feel flat.[^23][^13]

---

## References

1. [Claude Code Best Practices - GitHub Pages](https://rosmur.github.io/claudecode-best-practices/) - Write tests BEFORE implementation; Confirm tests fail (avoid mock implementations); Commit tests sep...

2. [Creating the Perfect CLAUDE.md for Claude Code - Dometrain](https://dometrain.com/blog/creating-the-perfect-claudemd-for-claude-code/) - Learn how to create the perfect CLAUDE.md for Claude Code and improve your development workflow with...

3. [Claude Code Best Practices \ Anthropic](https://www.anthropic.com/engineering/claude-code-best-practices?curius=2107) - A blog post covering tips and tricks that have proven effective for using Claude Code across various...

4. [Claude Code: Best practices for agentic coding - Anthropicanthropic.com › engineering › claude-code-best-practices](https://www.anthropic.com/engineering/claude-code-best-practices) - A blog post covering tips and tricks that have proven effective for using Claude Code across various...

5. [Best Practices for Claude Code](https://code.claude.com/docs/en/best-practices) - Tips and patterns for getting the most out of Claude Code, from configuring your environment to scal...

6. [Improving Claude Code Workflow with Living Documents - LinkedIn](https://www.linkedin.com/posts/mishamanulis_sample-work-plan-activity-7406694668492509184-NyBf) - Lately, I've been treating the Claude Code chat like a workspace and treating a markdown feature doc...

7. [An agentic meta prompt for Claude Code that creates powerful minimal workflows](https://www.reddit.com/r/ClaudeCode/comments/1le99rv/an_agentic_meta_prompt_for_claude_code_that/) - An agentic meta prompt for Claude Code that creates powerful minimal workflows

8. [How I Turned Claude Code Into My Personal AI Agent Operating ...](https://aimaker.substack.com/p/how-i-turned-claude-code-into-personal-ai-agent-operating-system-for-writing-research-complete-guide) - Analyze multiple research sources and create summary documents. Reorganize content folders or resear...

9. [Claude Code Best Practices: Lessons From Real Projects](https://ranthebuilder.cloud/blog/claude-code-best-practices-lessons-from-real-projects/) - Practical Claude Code lessons from shipping real projects: my setup, BMAD vs plan mode, how I struct...

10. [This Perplexity + Claude Workflow Turns Research Into ... - YouTube](https://www.youtube.com/watch?v=CUmXBN6BmDU) - This video wil show you how to use Perplexity + Claude to generate research and turn it into visual ...

11. [Tried Perplexity alongside Claude Code for quick research and ...](https://www.reddit.com/r/ClaudeCode/comments/1oez72e/tried_perplexity_alongside_claude_code_for_quick/) - Something I found unexpectedly useful was pairing it with perplexity ai for quick context gathering ...

12. [Board Game Design – System-Oriented Design - Level 99 Games](https://www.level99store.com/blogs/design-series/system-oriented-design) - A system is a set of interlinking rules, resources, mechanics, and components that work together to ...

13. [[PDF] System Thinking in Game Design - Proceedings](https://proceedings.systemdynamics.org/2024/papers/P1159.pdf) - Abstract. This article describes how system thinking can be used in advanced game design to make des...

14. [MDA framework - Wikipedia](https://en.wikipedia.org/wiki/MDA_framework) - In game design the Mechanics-Dynamics-Aesthetics (MDA) framework is a tool used to analyze games. It...

15. [MDA Framework - Deliberate Game Design](https://deliberategamedesign.com/mda-framework/) - The MDA framework, short for Mechanics-Dynamics-Aesthetics, is one of the classic tools for analyzin...

16. [Game Design Systems That Improve Player Retention and ...](https://www.linkedin.com/pulse/game-design-systems-improve-player-retention-engagement-p99soft-mbkvf) - This article breaks down the game design systems that actually drive player retention and engagement...

17. [How to Design the Mechanics of Your Board Game](https://brandonthegamedev.com/how-to-design-the-mechanics-of-your-board-game/) - Every single developer has different methods for creating their games. This article is the third of ...

18. [How To Perfect Your Game's Core Loop - GameAnalytics](https://www.gameanalytics.com/blog/how-to-perfect-your-games-core-loop) - Learn how to fine-tune your core loop to enhance your gameplay. We take a deep dive into the main ac...

19. [Building an Ontology of Boardgame Mechanics based on ...](https://www.sbgames.org/sbgames2017/papers/ArtesDesignFull/175272.pdf)

20. [[PDF] Building an Ontology of Boardgame Mechanics based on ... - LUDES](https://ludes.cos.ufrj.br/wp-content/uploads/2017/11/Game_Mechanics_Ontology.pdf)

21. [Game mechanics - Wikipedia](https://en.wikipedia.org/wiki/Game_mechanics)

22. [Shell's taxonomy of game mechanics - Practical ...](https://www.oreilly.com/library/view/practical-game-design/9781787121799/8cceb529-abde-4755-be15-4e021b5c4763.xhtml) - Shell's taxonomy of game mechanics Jesse Schell (in The Art of Game Design) has done excellent work ...

23. [How Game Design Principles Drive Player Engagement](https://www.cgspectrum.com/blog/game-design-principles-player-engagement) - This post will define the principles of game design and explain how they enhance player engagement, ...

