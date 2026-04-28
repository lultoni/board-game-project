What Gamelogic do I need to worry about:

Board

- made up of 12 x 12 tiles

- equal terrain spread

Tile

- Has one of four terrain types

- piece on it (or not ofc)

- Effects that skills apply (timer for how long they stay in amount of
  turns)

Pieces

- different base types of pieces (guard, king, champ)

- if is a champ, they have a class

- 3 skills if king, champ

- terrain they are good on (neutral for guards and king)

- health, armor, other buffs/nerfs from skill-effects etc

- range is 2, for king it is 3 (minimum is 1 again)

- 2 hp normally

  - if hp drops to 1

    - speed is capped at 1

    - range is lowered by 1

  - armor will not stop hp debuffs

- if damage is dealt this is the logic:

  - shields

  - temporary armor

  - armor

  - temporary health

  - health

Actions the pieces can take

- Move

- Attack

- use skills

Skill Runes for both players

- Both start with 5

- every turn add rune_tally

- rune_tally starts at 1 and increases by 1 every 5 turns (5, 10, 15...)

Which Turn is it

- Timer starts at 1 and for each phase goes up by 0.25

- movement phase

  - 2 move slots, which do not have to be spent

  - minimum speed is 1

  - guards have 2 speed

  - moving onto filled square you attack that piece

- action phase

  - 2 skill slots at start of the game

  - this increases every 2 rune_tally increases

  - again, skill slots can be used but do not need to be

    - Do you want to add a turtle bonus?

Using a Skill:

- Can you afford the skill? (grey it out otherwise)

- the target has to be in range

- skills have tags for checks that have to be done

- offense skill, 1 damage

- damaging terrain effect, 1 damage when

  - piece enters the affected tile

  - casted on a full tile

  - pieces turn ends on tile, and it has not taken damage from terrain
    effects that turn

Attacks

- Deal 2 damage

- if champs attack they take 1 damage themselves

- if no death of piece, stay on original square

- surrounding guards absorb the damage taken if a champ is attacked

  - closest guard to the attacker is chosen, based on you know what

Game Stages

- Pre-Round: Flip Coin on who goes first

- Pre Round: Pick and Ban Phase

  - per "turn" actions here: 2 Ban, 2 Pick (Picks go into slots, that
    are locked afterwards)

  - if less slots free than picks, then you just pick the remaining
    champs

  - Or instead of locking, both players confirm after a short
    readjustment window of like 10 seconds or so

- Playing a Round

- Round Ends

  - One of the kings dies

  - only the kings are left

  - no pieces are taken in 10 turns
