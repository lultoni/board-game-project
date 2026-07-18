# Playtest 6 - Elias vs Dorian - Stack M (Game Length Cut)

*Digital prototype, online play, 2026-07-09. Raw game log exported at end - telemetry answers everything factual. This file is for feel, judgment, and things the log can't see.*

*Stack M under test: board 8×8, Armor max 2, Injured has no penalty, no draw conditions, Steal costs 4, Combo Bonus also triggers on movement-causing skills.*

---

## Post-game - Elias

### Feel

**Game length feel** - way too short / a bit short / just right / a bit long / way too long.  
Comment: i felt it was the right length, could be a bit faster, but it depends on if the game is played on a board or a table and if both players really think about moves or if they are pressured by time

**Game shape** - one clear climax → game ended / two climaxes / multiple climaxes / no real climax.  
After the decisive moment, did the game end naturally or drag? 
Comment: mmh i would say it depended really. because what was happening sometimes is that when you have the first exchange with guards, that both players retreat a bit and wait up until the economy catches up and they can do cool stuff again, so you had potential down phases, but it could also be tense right until you reach the end game (and decide the game there). I just had an idea: we remove moeny from the game and just punish players who spam skills (or the same skill - aka: my idea of "multiple skill activations from the same piece in the same turn deal 1 dmg to that piece")

**Mid-game stalling** - did the "Armor stack + nothing happens" pattern reappear? Yes clearly / briefly / not really / not at all.  
Comment: well it _could_ happen (as it does with engines playing themselves) as the person who breaks the standoff usually is in the disadvantage (losing a piece more at the end of trades), but it is not bound to happen - it again depends on player behaviour, but the most optimla strat that seemed to emerge seemed to be stalling it out. armor was still a very present thing inside the game, but it was never directly "only stacking armor on passive", rather it was "wait up so i have money again to attack".

**Board size (8×8)** - cramped / a bit tight / just right / a bit open / too sparse.  
Did pieces feel like they had enough escape options? Yes / sometimes / no.  
Comment: i felt this board size to feel really natural and perfectly fitting for the pieces to move around and create interesting positions

**Armor cap 2** - meaningful but not dominant? Did the lower cap change how you played?  
Comment: i mean it was easier to get pieces of the board that had been "maxxed out" than before, but it was still a huge and centric part of the gameloop. you still needed it to survive and not just lose all your pieces

**Injured has no penalty** - right call (cleaner) / mostly fine / a bit weird / meaningless.  
Did you treat an Injured piece any differently from a Normal piece? 
Comment: it felt normal. it was one less thing you had to worry about. it meant that just because a piece was injured it did not mean it was out of the fight

**No draw conditions** - any point where you'd have wanted a draw but couldn't? Game ended naturally / wanted to draw / was a problem.  
If wanted draw - why? 
Comment: in our games we did not feel it that we had situations where we needed a draw - it was rather that at some points we would have wished a "give up" button

**Steal cost 4** - must-pick or finally tunable? 
Comment: it finally felt fair, altho now it was overshadowed by tempest, because it was same cost, but tempest felt like it had more impact with moving so many pieces around (most likely comes from us not being advanced enough to know how to manage the economy well yet and shutting down an opponents economy)

**Combo widening (movement skills tick counter)** - felt intuitive / took getting used to / confusing / did not register.  
Any skill (Tempest, Hook, Blast, Shove, Swap) feel especially strong because of it? 
Comment: it felt like it was finally very usable - the combo system. before it was a one trick pony and now its very versatile. now tempest was very good tho for strategies who focused on the combo system. BUT it was not the single dominant strat, hook+lance is still a good cheap combo that in one game here almost beat me who had a big focus on trying to maximise combos and use diverse skills (this is good!).

**Per-turn complexity** - much lighter / lighter / same / heavier / much heavier than prior play.  
Ever feel overwhelmed by the option count? Yes / sometimes / no.
Comment: i would say its the same - pretty complex, but still feels managable

### Timing feel (log has the numbers - this is whether they felt right)

**First Guard death** - too early / **right time** / too late.  
**First Champion death** - too early / **right time** / too late.  
**Combo bonus triggers** - landed at earned moments / **a bit easy** / a bit rare / something else: 
**Injured pieces surviving multiple rounds** - **right** / a bit off / very off.

### Systems

**Skill drafting** - fair, engaging? Did you draft with specific pairings in mind? 
Comment: it was fair and enganing, altho it can take a bit of time

**Turn flow** (Move + Skill phases together) - very intuitive / mostly yes / sometimes confusing / often confusing.
Comment: i personally feel like its good, but dorian said that he would really wish to move more pieces during movement phase

**Skill balance** - anything too strong or too weak (besides Steal)?  
Too strong: focus Too weak: push (so the one pushing a piece 1 tile away)
Best combo you pulled off: tempest into killing one of the shoved pieces, for dori most likely hook+lance

**Bodyguard** - did you actively reposition Guards to use it? 
Comment: yes, with the tighter psotiosn it happened more often, but it did not feel like the sole focus inside the game because the guards fell of the board very quickly (too quickly i feel like imo)

**Money spending** - always more to spend than Money / balanced / sometimes nothing to spend on / often sat on Money.  
If sitting on Money - why? 
Comment: i feel like the games were slowed down by not having enough money. again is part of us not being perfect players, but still

**Favourite moment - what happened?**  
realisiting that you found the winning move in a position or that you found a clever way to not lose in a position which looked like a bad position at the start
aka: finding cool and clever moves in a single posistion (and with moves i mean "full own turns")

**Opponent's game** - same puzzle / bit of both / direct contest / hard to say.
Comment: hard to say. really hard to say - it was more about having a cool time together, but i would lean a bit toward direct contest. you most often only really "puzzled" your own position

**Most confusing or frustrating rule or moment?**  
Comment: that the combo system had a bug and did not work as intended - this lost me a game. i feel like in the future for every rule i want to make a document where every possible situation and edge case is wrote down and then i say what should happen and we hence make test cases for the engine to be forced to pass

### Ratings

- Game length: 1 (way too short) - 2 - 3 (just right) - 4 - 5 (way too long): 3
- Pacing / single-climax shape vs prior play: much worse / worse / same / better / much better: pretty similar
- Overall enjoyment: 1 - 2 - 3 - 4 - 5: 4.5 (playing with friends really makes it better)

---

## Post-/During-game - Dorian

you need a tutorial / learnign / help section where you can read up on skills or rules during any screen (new button next to settings)

the player who loses a piece gets money

time based mode for MP, like a competitive mode. also already ticks time in draft or you have a seperate draft clock maybe.

the premade loadouts are not mirrored correctly

automatic turn ends for humans on that being the only legal move

combo counter is broken - when you cast blast and then from a different pice do lance twice it only gives the combo bonus once instead of actually giving it twice. i think i really need to rethink how this works in detail here in a brainstorm session so it matches my mental model

im early game will ich constantly was für 3 skill aktionen machen, was aber noch nicht geht

when opponent is in sandbox and i make a move (in mp) we get the anti-cheat notification that the opponents engine disagreed

we need a move history to look through old moves in the match screen

focus cost increase +1 (so it costs 2 now)

we need a on hover for the in between step in move attacks when click-moving

es gibt keinen weg aufzugeben und so

ein chat wäre auch ultra funny, was das online viel lustiger machen würde

‌

---

‌

feedback von dorian:

game länge: durch sandbox kann es sehr lange dauern, aber ansonsten okay (maybe sollte es einen weg geben die actions aus der sandbox umzusetzen (had einen check, dass das nur geht, wenn es eigene aktionen waren oder sowas)

er fand es sehr nervig, dass man nur 2 figuren bewegen konnte pro movement phase

stalling? eigentlich nicht - man muss halt aktiv spielen und das muss man bei 8x8

zu komplex? ich sag es geht, aber wir sind jetzt beide auch nicht “das maß aller dinge” (aka keine profis haha)

skills zu op? nur focus, der rest ist okay

bodyguard? kam nicht so direkt ins spiel weil ich zu greedy war, aber sein fine

war es gleiches puzzel oder direkter contest? es geht in die richtung schach, man kann es deep machen und “ich versuche krass zu sein” aber man kann auch einfach draufhalten und hoffen.

also fühlt es sich nicht an wie ein puzzle was manv ersucht zu lösen? ne, würde ich jetzt nicht sagen

wie viel spaß hattest du overall von 1-5? mit freunden ist es richtig geil. ist halt nichts was ich alleine spielen würde (bin ich nicht die person für). ich sag ne 4 von 5.

ab mitte des games fängt halt das standoff richtig an. weil dann die person die den ersten move macht verliert.

aber fokus braucht einen teureren preis, das hilft schon viel.

---

zeitzen unserer spiele (durch das digitale spielen geht es aber auch generell schneller - bissle wie ein apfel birnen vergleich, aber still):

- regeln erklären: 10/15 minuten (wir haben gesagt, dass wir das durchs spielen herausfinden)
- spiel 1 (first game loadout): 45 minuten
- spiel 2 mit custom draft: 1 stunde
- spiel 3 mit custom draft: 1 stunde bis 1:30 stunden

---

## Standoff-resolution follow-up

*This game is the load-bearing evidence for the pending draw-condition rule change (Direction B, 2026-07-08): 5 rounds without capture after round 10 → loss for player with more pieces; tiebreaker on Champion count; then genuine draw. Deferred until this playtest.*

**Did an infinite-standoff pattern appear at any point?** No / briefly / yes.  
Was there a moment where either player felt "I could just wait and nothing bad happens to me"? 

**If standoff appeared - what broke it?** Someone committed / one side ran out of a resource / draft had a way through / never broke - _______  

**Given what you saw: does the proposed 5-rounds-no-capture rule feel like the right shape, or does the game need something more structural** (asymmetric tempo pressure, board-control scoring, etc.)?