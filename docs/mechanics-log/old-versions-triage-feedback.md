# Old Versions Triage — Feedback

*Write your notes under each idea. When done, bring this back to Claude to update the triage and living docs accordingly.*

*Full context for each idea: [`old-versions-triage.md`](old-versions-triage.md)*

---

## New / Possible

### Piece & Roster

**King has 3 skill slots** (not 2)
> this could make the king into a piece that is powerful, hence it is still risky to walk forward with it, but you would be rewarded with a strong piece.
> on the other hand you would have the possibility of running into the issue that the king could become the ultimate "stay back support" -> healing and buff skills
> could be tested post-v1 like you said

---

**Spirit Mage as named win-condition piece** (flavour / King identity)
> this is for the design itendity phase when we try to develop the identity of the game (so not the mechanical feel of it)
> also a post v1 thing to do (maybe even post post-v1-tuning)

---

**Full Champion roster naming** (Blacksmith, Necromancer, Bard, etc.)
> i mean the current design is that every champ is like a clean slate/tempalte and that is somehting i don't want to change
> if we were to give the champs a identiy beforehand then it would destroy this "mental freedome to do anything"
> what _is_ possible is that we make the skills in such a way, that when they are on a piece that that piece becomes like a mixture of such "champ identities" - this could be something like making the rune theft into a "vampiric suck" or something and then designing the pieces and the physical skill slots on the pieces into a way to then show some kind of vampiric energy

---

### Board & Terrain

**Terrain stat modifier system — damage/cost neutral** (if terrain ever returns)
> yes can be explored if we want to go into this direction but currently due to cognitive load we do not add in terrain (also becuase it locks in the way users play and position themselves, making games feel similar on the same map because there is then "one way to play/win")

---

**Mirror board / FEN seed generation** (if terrain returns as map variant)
> same as above

---

### Skills

**Attack-then-reposition on single activation** (Ambush, Hakenzug, Klingenorkan)
> yes this is definitly worth exploring, but i feel like this is a "over time" thing as adding in way too many skills makes it more about balancing skills instead of the other game systems

---

**Move Slot loss as a debuff** (target moves one fewer piece this turn)
> this feels very op because it restricts the opponent so much
> maybe with more discussion it could be thought about but this has no prio rn

---

**Temporary Armor** (absorbed once, then gone)
> becomes something new to track mentally - this is generally a thing with temporary effects that it is hard or rather straining to keep track of
> if we add in a way to easily keep track of temporary effects then yes it could be something worth looking into - maybe a research topic on how board games so such things
> but this is also only worth it if we think about what game feel it would bring or which game feel it could improve (a brainstorm session)

---

**Armor Destruction as explicit offense category** (Pocket Thief, steal/remove Armor)
> yes could be interesting to explore this but i think it could also lead to never really using armor (dead skill) because there is always this threat of massivly losing to this (if these new skills would be too strong)

---

**Active Guard-Bind** (Wächterband: Guard loses move, ally gains +1 Armor)
> this sounds like a temp move block for guards again - cool idea i suppose but again how this can be implemented would have to be seen

---

**Information / scouting skill** (Runenblick: reveal Rune count + optional steal)
> "in a perfect information game this is unusual" - in a perfect information game this is irrellevant?? we literally know everything

---

**Movement as a skill** (Schnelltritt, Escape Plan / Rückzugsplan, Pferdesprung)
> we already have this

---

**Speed boost via skill** (Eagle Vision: target gets +1 Speed next turn)
> yes could be a worthwile idea to look into
> the question is how this would be different from move skills and how this would make other ones of these maybe redundant (so we look which gaps would actually be filled by adding this and which currently available options get overshadowed by this)

---

**Heal at range** (Plague Medicine: heal Injured on Skill Path, not just adjacent)
> yes worthwile exploring, but this here has the question of cost compared to other options or skill combos or something

---

**Free-direction push** (Federstoß: push to any direction, attacker chooses)
> we already have this in one of our skills (forgot name rn but this exists)

---

**Line pull** (Strömungsruf: pull all enemies on a line toward its centre)
> very intersting idea - the execution would just have to be elegant enough so its not a "10 thousand exceptions/edge cases in the rules" but just a simple rule of thumb basically, but otherwise i think this could be cool yes

---

**Range empowerment category** (Bardic Inspiration, Blood Spear, Fokusstoß, Klingenruf)
> we already have one skill for this with focus strike
> this could definitly be expandend tho with more skills (again: skill cannot be too op or make other ones redundand)

---

**Shield duration system** (shields expire by turn count, not damage — Mirror Shield reflects)
> this again is the question of keeping track of temp things
> also: the economy for this has to be balanced very well because if its too expensive then it is never worth spending on temp armor - if it is too cheap then the opponent will never be able to break through it

---

### Economy & Progression

**"Shortfall never closes" as explicit design goal** (players can never fill all skill slots)
> i would say this came from having so much need to use skills but not having enough runes to use them i think
> this is something to look out for when we design our economy oursleves (do we have a point in our docs where we have "things to look out for"?)

---

**Rune cap** (e.g. max 8)
> pros: you cannot hoard runes
> cons: you cannot do expensive rune combos later on in the game - forcing you to use low rune combos
> i think the cleanest fix for rune hoardning (because of various reasons) is to trough other design decision that encourage effective skill usage/rune spending instead instead of _forcing_ them to be spend unproductivly or just fully losing out on runes (automatic falling behind if you don't spend them "for no reason" (better invest bad then lose the money entirely))

---

**Skill cost calibration: average 2.5–3 Runes across catalogue**
> i think we shouldn't math this out - i think we should see how it **feels** to use skills and if you never feel like "damn i don't have enough runes to do stuff" but also never "damn i can't do anything" or even remove the incentive of spending strategically because you have too many
> difficult topic, but i think it will cristalise with more games played

---

### Turn Structure

**Turn counter as global timer** (0.5 increments per player turn — "turn 7" = both acted 7 times)
> das ist ja so ähnlich in schach auch - das kam vorallem davon für temp effects eine gute weise zu finden diese zu tracken
> für ein brettspiel selber sage ich ist das nicht so krass wichtig so präzise das zu tracken. die runenzahl ist ja eigentlich nebensächlich

---

### Drafting

**Class-based skill pool** (Champion's class determines which skills you can draft)
> i again think this just limits the strategy freedom the players have
> upside: it simplifies the complexity of skill drafting a bit - but again: slighty lowers the choice freedom of the players (limits the space of posibilites)

---

**One Champion per terrain type constraint in draft**
> we don't have terrain systems in place but if we ever do then we can surface this again

---

### Win Conditions

**Draw if only Kings remain**
> interesting idea. i think we could test this but because it needs endgames with only kings remaining we could put it in the back pocket if only-kings-left endgames become common (and then don't feel fun)

---

### Meta / Design

**Piece compatibility / adjacency synergies** (adjacent ally bonuses, class combos)
> yes i love synergies because i think they reward complex setups and positioning well (like we want to encourage)
> the way this is done is a very interesing prospect to be explored/researched

---

**Use ROE AI evaluation factors as a playtest lens** (Guard protection, LoS sheltering, etc.)
> yes on one hand it could be interesting to have an ai-opponent/ai-eval, but on the other hand then the game will feel solved and thinking of solutions will not be as fun anymore because you know that the computer could just do them better than you

---

---

## Deferred — Related

*These are already tracked. Notes here = override or update the existing tracking.*

**Draft from pool, then assign to Champions** (OQ-35, deferred post-Layer 3)
> 

---

**Flexible piece placement / reveal-style simultaneous** (OQ-36 + OQ-48, deferred post-Layer 3)
> 

---

**Pick-and-ban Champion draft** (OQ-35 variant, deferred post-Layer 3)
> 

---

**18 Champions to draft from** (OQ-27, deferred post-v1)
> 

---

**3 Move Slots** (deferred — may be superseded by AP system Layer 4)
> 

---

**8×8 board** (OQ-1, Layer 5)
> 

---

**Checkmate-style win condition** (OQ-19, Layer C)
> 

---

**Unified AP system** (OQ-26, Layer G)
> 

---

**Hex board** (OQ-42, reopened — needs research first)
> 

---

**Cascade trigger: kill → free skill slot** (OQ-51 candidate)
> 

---

**Positional payoff: haven't moved → +1 Range** (OQ-51 candidate)
> 

---

**Rune Theft balance** (OQ-34, monitoring Layer 2)
> 

---

**Minor/Major skill slot cost** (OQ-50, deferred — design ultimate skills first)
> 

---

---

## Archived / Similar

*These are closed. Notes here = reopen something, or confirm the closure.*

**Terrain system** (removed ADR-001/002)
> 

---

**AoE skills** (Tremor 2×2, Inferno 3×1)
> 

---

**Persistent tile effects** (death zones, blocked tiles)
> 

---

**Spell reflection** (Aerial Shield / Mirror Shield)
> 

---

**Rock-Paper-Scissors element advantage**
> 

---

**Champions take self-damage on standard attacks**
> 

---

**Mutual kill on Guard-vs-Champion melee**
> 

---

**Active Guard-Bind** (already in New/Possible above — skip if noted there)
> 

---

**Linked movement** (move to act — withdrawn Session 1)
> 

---

**Performance-based Rune gain** (closed OQ-47)
> 

---

**3 HP for Champions/King** (scrapped OQ-18)
> 

---

**Retaliation mechanic** (overextension → opponent gets retaliation action)
> 

---

**Raise Undead** (10 Rune resurrection)
> 

---

**War Cry** (all skills +1 Range next round, 5 Runes, global tracking overhead)
> 

---

**CR-style draft** (strict single interleaving — closed OQ-43)
> 

---

**Ban phase in skill draft** (closed OQ-44)
> 

---

**Narrative / lore** (Rabbit, Primordials, Advisor NPC — reserve for art phase)
> 

---
