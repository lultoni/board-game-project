\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-- Backend Logic
\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\--

use events and animations, only after an animation is done call an event
that changes the correct backend things (so what the player sees and
what the computer sees are different things)

find a way to implement a battle system that takes in 2 players and then
animates the scene (and does the action selection) according to if the
player is real or an ai

how can you sync both sides in a network (self no delay on actions,
others yes)

-\> what about running everything frontend on the client and everything
logic on the server (and client, so only sending actions that are legal)

\--\> question: event calls on a server so the updates are done for
everyone (0 delay)

turn states ((is before every) turn, moving, selecting tiles, selecting
skill\...)

-\> turn order

ai player decision tree, ai player with hyper knowledge to learn about
the game

http://blog.gamesolver.org/

For an ai player: set weights to aspects that a eval function evaluates,
which change per agent, that will play against each other -\> the best
weight set wins (if it looks good) to be the value network to evaluate
positions

FEN needs to include phases and how many of the actions 1 players has
already used

-\> update how the moves are shown (So something like Kb6)

\--\> show the tile names on the bottom right corner

Internal counter for how many (skills) of each class the ai has to
choose a balanced strategy

-\> this can vary for each ai opponent

\--\> make it learn from your choices and counter pick from the data or
self added rules

Compare all of the agents/versions on random positions on how fast and
good the moves are

The evaluating/scoring is only based on the piece values

-\> each thing is based on the piece it is part of

-\> So what is the score of all the pieces (do all of the champions have
the same max score? And what is the guard max score?)

The random positions can be created from the position fen with rules to
make them legal/at least half real

-\> These random positions need to be pretty much equal

Things to score on for the Agent:

- Accuracy

  - (is this close to the best move: needs to be calculated by the
    minmax? -\> d1 and then look from the best moves for d2 and so on)

  - Minimax/Negamax with ab pruning would be best if you find a way to
    not generate all possible moves, but generate one, evaluate, gen the
    next and so on **(COMBINE GENERATION WITH EVALUATION)**

  - Move order can look (most likely) only at these things

    - Quick note: move order is best with the note above

    - which pieces take actions

      - so champion actions are valued higher than guard actions

      - the closer a piece is to the middle the higher its priority is
        in the heuristic to determine move order

    - which combination of actions is the best

      - m, ma, maa, mm, mma, mmaa...

      - all of these can/have to be sorted

        - maybe base this off of the runes and/or the move count

  - transposition table to look for already computed moves (again best
    with partial generation)

    - You can change the size of the entries or the space the entries
      take up to make the transposition table better

    - [[Part 11 -- Optimized transposition table - Solving Connect
      4]{.underline}](http://blog.gamesolver.org/solving-connect-four/11-optimized-transposition-table/)

    - [[Part 12 -- Lower bound transposition table - Solving Connect
      4]{.underline}](http://blog.gamesolver.org/solving-connect-four/12-lower-bound-transposition-table/)

  - null window search

    - narrow ab window to quickly just say if a move will make the
      position better or worse (i think)

    - [[Part 8 -- Iterative Deepening & Null Window - Solving Connect
      4]{.underline}](http://blog.gamesolver.org/solving-connect-four/08-iterative-deepening/)

  - anticipate losing moves, so they will be pruned

    - in Pascals version he has a possible() func that gives back all
      the possible moves

    - instead he now uses possibleNonLosingMoves() func that doesn't
      consider losing moves which saves a little bit (not that much i
      think)

- Execution time (on the same machine)

- (Number of explored moves)

- Each of these can be weighted to score each ai or you just look at all
  of the ai's and rate them yourself

If you really implement all of the bitboards then you need to really use
them for everything: checks, moves, everything

\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-- Frontend Design
\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\--

make overview for gameflow (basically DEA with each state being
something that either describes the screen or the player activity (world
map/battle)

-\> this can help with designing the UI

sound/music design

-\> how does audio work in unity (usage of events to trigger not only
animation but also sound effects)

give the pieces character that the players can resonate with

5 stack of skill runes, like in orlog; also take a look at the camera
angle of orlog (the runes to be added float above/to the side and then
move toward the stack (orlog animation for rune aditions?))

sprites to change: air mage, earth mage, water mage Dreizack

visual queues: timer von Effekten als pie diagram, Skill range als
weißen ring (on hover), wo gut als fade (on hover) , Skill effects
(defense icon, maybe the other two ones?)

display last made actions (Spur hinterm Dreizack)

Generate Images with AI for concept art

Game timer

Zwerge, Elfen, Orks

Finde einen Weg (Armor, M Range, S Range, Attack, (Good Terrain))
anzuzeigen pro Piece

Icons für die Champion Klassen die bei der Skill Auswahl angezeigt
werden

-\> piece Wahl oder immer so wie armor zb

-\> überarbeite den weg wie die skill Auswahl aussieht

\--\> mehr Infos zu den skills und nicht bei Auswahl erst (hover so ein
info window oder doch nur so 2-3 basic Infos über (naja außen rum)
klassenring)

what is the color scheme of the two sides (chess like?)

-\> Outline for the pieces

When click next phase thing safe clicking and changing all the timers
(.25 seconds)

Change the orientation of the sun based on the amount of pieces on the
board (slow moving so the players don\'t notice/think about it)

Create Icons for all of the classes

Do you really wanna do pixel art?

\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-- Gameplay Changes
\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\--

what is happening when you win/lose a battle

-\> best of 3? (shorter combat when this (same pieces?))

-\> story mode vs. online mode

what do you want to do with the terrain, surely you can\'t keep playing
the same fucking map like right now man, like let the players choose or
build their own

-\> how can you implement this in the story mode

-\> strike a balance between map knowledge and freshness/consistency

\--\> look at btd6 with all the different maps and different, but
similar approaches

How can I motivate players to play my game for longer or at all

-\> multiplayer: leaderboard to compare self to others

-\> singleplayer: rougelike esque with replayability

\--\> base story with tutorial and such

\--\> repeatable part (does not have to be too major, look at jump king)

\-\--\> battle tower, how far can you come? (Pokémon rogue)

\--\> count up some arbitrary value, maybe a currency

Give the player unexpected rewards for playing the game

-\> finishing the story mode: unlock adjustable colors

\--\> win a story battle: unlock new color/piece skin/player outfit idk

Pickrate, Winrate (Champ v champ)

\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-- Lore Ideas
\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\--

What is the name of the Game and what is the name of the game the lore
persons play? Is it the complete same?

Make your own gods based on Germanic/Nordic Religion

-\> maybe fight them as end bosses/opponents in the story mode (summon
god x - \"you have called me to\... play (GAME NAME)? Really?\"

\--\> Ascend to godhood or something after beating the game

\-\--\> online play is just pitching (demi-) gods and looking who is
stronger (leaderboard is maybe a reference to this)

-\> the entire fucking universe is centered around the game

\--\> each god made a world/race to find the best player (the chosen one
or something)

\--\> you fight for your god

Der Spieler steuert die Figur, die das spiel spielt und im story mode
ist das finale die Figur gegen den spieler.

-\> Figur hat genug herumgeschaut zu werden (Spieler = (ein) Gott)

Think about the concept of the game (NAME PLEASE) being so intertwined
with the world building, why and how can this be? What are the effects
on the general population? Do they play it? Do they despise it? Do they
see it as a way out of poverty?

should the figure have a lore rival like in Pokémon?

What is the story of the pieces? That influences Design

There are other races besides humans

Meschen, Zwerge, Elfen, Orks

-\> H, D, E, O

Name of the game:

broad genre & perspective (gears of war) battle/tactics/strategies,
commander/god/king/ruler

positive words (or avoid verbs that sound like work) advise, blabble,
battle, beat, bleed, breath, challenge, command, compete, develop, dive,
expand, explore, fight, grab, hail, hammer, help

verbs (can describe pace, ghost runner)

juxta positioning (nuclear throne (nuc new, th old)) throne, rune

cool word and uncool word (black desert, desert child) magic cheese,
game of gods, power terrain, champion soup, skill doodle

don\'t try too hard of course

alliteration (prince of persia)

intrigue

subconsious assosiation chess (chase), crusader kings, slay the spire

\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-- Possible Skill Effects
\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\-\--

Remove Piece from Board (Weakened)

Terrain Version

Stop Usage of Moving, (Attacking) & Skills

Stop Usage of Moving

Stop Usage of Skills

Terrain Version

Move Piece in Direction (Own, Opponent or All)

Move Piece Back

Move Piece Forward

Move Piece away from Target (All Directions)

Move Piece towards Target

Immunity for Skills, Attacks

Immunity for Skills

Immunity for Attacks

Reflecting Skills, Attacks

Reflect Skills

Reflect Attacks

Blocking Space

Give Piece Good Terrain Effect

Terrain Version

Give Piece Range + 1

Terrain Version

Switch Pieces (Own, Opponent or All)

Switch Only Skill Using Piece with other Piece

Sacrifice Own Piece for Boost

Sacrifice Own Piece for Runes

Bring Back Own Piece

On a timer

Bring Back Own Piece for Debuff

Take Form of another Piece

Take Skill from another Piece

Create Decoy Piece with a set amount of Actions

Permanently Lower Cost of specific Skill

Heal Piece/Give Armor

Buff Damage

**How to show effects:**

1.  Tile

    a.  Terrain, Damage Effect, Blocked Effect, Artifact (Paladin),
        Lookout Tower, Sirens Light

    b.  Timer for how long the effects stay

    c.  Think about the 3rd dimension if you do 2.5D

2.  Tile Highlights

    a.  Good Terrain, Possible Move/Skill Target (Piece vs. Tile)

3.  Piece Overlay Top

    a.  Armor Icons (Enabled, Disabled, Temporary), Shield Icons
        (Amount/Timer), Actions Left, Is On Good Terrain

4.  Piece

    a.  Injured, Healthy, Both as Moved (?)

5.  Piece Overlay Bottom (?)

    a.  Movement Range, Skill Range Modifier, Skill Damage (Dealing,
        Receiving), Can Attack

    b.  Only Display when Modifiers are not 0 (and when they are
        disabled)

6.  In Skill Menu

    a.  Skill Infos (Which ones?)

**Main question: How do I want to display timers?**

Timer for effects/boosts:

Good Terrain, Skill Range, Temporary Armor, Deactivated Armor, Shields

**Names of the Game:**

Champion Chess 3.5 4 =7.5

The King\'s Tactical Konquest 4 2.5 = 6.5

Of Champions and Shit 2 2 = 2

Skill Scramble 2 3.5 = 5.5

Champs' Gamit 3 3 = 6
