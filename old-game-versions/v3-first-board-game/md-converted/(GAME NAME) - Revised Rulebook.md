### **(GAME NAME) - Revised Rulebook Draft**

#### **Game Concept**

Two players command an army of Guards and Champions, led by a King. They strategically maneuver their pieces across a board made up of different terrains, utilizing strategic skills to outwit and overpower their opponent. Victory is achieved not by sheer numbers, but by tactical superiority and the decisive capture of the enemy King.

#### **Goal of the Game**

The first player to capture the opponent's King wins the game.

A **Draw** is declared under one of the following conditions:

- No piece has been captured by either player for 10 full rounds.
- The only pieces remaining on the board are the two Kings.

#### **Components**

- **Game Board:** A 12x12 grid composed of 36 tiles of each of the 4 terrain types.
- **Player Pieces (per player):**
  - 1 King piece
  - 5 Champion pieces
  - 6 Guard pieces
- **Skill System:**
  - A pool of Skill tokens to be chosen from.
  - Markers to place on Champion pieces to indicate which skills they have equipped.
- **Resources:**
  - Rune tokens (currency for skills)
- **Trackers:**
  - A Round tracking device.

### **Setup**

- **Board Construction:** Players assemble the 12x12 game board. The terrain tiles must be placed so that one 6x6 corner is congruent to every other corner, ensuring a balanced playing field.
- **First Player:** Flip a coin to determine who is Player 1.
- **Skill Selection Phase:** Player 1 chooses 2 Skills for their first Champion, Player 2 does the same and this is repeated until all Champions and the King have two skills equipped.
- **Piece Placement:** Players place their King, Champions, and Guards on the board: Champs go in the middle of the last row with the king in the center right, one row above them the Guards are placed.
- **Starting Resources:** Each player begins the game with 4 Runes.
- **Begin Game:** Player 1 starts the first Round.

### **Gameplay**

**(GAME NAME)** is played in **Rounds**. A full Round consists of Player 1 taking a **Turn**, followed by Player 2 taking a **Turn**. A player's turn is divided into a Movement Phase and an Action Phase.

#### **Resource & Progression System**

- **Runes:** At the end of each Round, both players gain 1 Rune. Starting on Round 7, this amount increases to 2 Runes per round. On Round 14, it increases to 3, and so on every 7 rounds.
- **Skill Slots:** Players start with 2 Skill Slots. Starting on Round 10, this increases to 3 Skill Slots. On Round 20, it increases to 4, and so on every 10 rounds.

#### **Movement Phase**

- You have **2 Move Slots**.
- Guards have a base speed of 2.
- Champions/Kings have a base speed of 1.
- You can use one slot to move one piece in any direction (horizontally, vertically, or diagonally), the distance of tiles moved is linked to the piece's speed.
- Terrain and injuries can modify movement speed. The minimum speed is always 1 tile unless a piece is immobilized.
- You may choose to use two, one or zero Move Slots.

#### **Action Phase**

- You have a number of **Skill Slots** (starting at 2).
- You can use these slots to activate the Skills equipped on your Champions or King.
- See the **Skill System** for details on Skill costs.

### **Core Mechanics**

#### **Attack System**

- A standard **Attack** is performed by using your movement to land on a tile occupied by an opponent's piece. The opposing piece takes 2 DMG and is removed from the game if it had no Armor, and your piece occupies the tile. If the piece was not taken your piece will not move.
- **Bodyguard Rule:** If you declare an Attack on a Champion or King that has a friendly Guard on an adjacent tile to both the attacking and defending pieces before the attacker begins their mov., the Attack is intercepted. Your attacking piece does not move, and a Guard is removed instead. If there are multiple Guards in Question the defender is allowed to pick which of them are removed.

#### **Health System**

- All pieces have 2 Health Points and can be in one of two states: **Normal** or **Injured**.
- Dealing 1 DMG to a Normal piece makes it Injured.
- Dealing 1 DMG to an Injured piece removes it from the board.
- **Damage Chart:**
  - **Standard Attack:** 2 DMG (removes a Normal piece instantly).
  - **Offensive Skill:** Typically 1 DMG.
- **Effects of being Injured:**
  - Movement range is reduced to a maximum of 1 tile per Move Slot (even for Guards).
  - Skill Range is reduced by 1.
- **Armor:** A piece can have up to 3 points of Armor. Each point of Armor absorbs 1 DMG from a single instance of damage and is then destroyed. Armor does not negate the negative effects on an already Injured piece.
- **Healing:** Skills can heal a piece, changing its state from Injured back to Normal.

#### **Skill System**

- **Skill Path:** For Skills that require a path (e.g., moving a piece, damaging a distant target), the path is determined like a **Queen's move in chess**: any number of **unobstructed** squares in a straight or diagonal line within the skill's range.
- **Skill Cost:** Each Skill has a set cost of Runes, which has to be paid before the skill can be used.
- **List of Skills:** This can be found at the bottom of the Rules

#### **Terrain System**

When a piece stands on a terrain tile, it gains the following effect:

| **Water**     | +1 Skill Range. (Has no Effects for Guards)                                                                                                                          |
| ------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Forest**    | Gain 1 Temporary Armor upon entering the tile. This armor is lost if the piece moves off the tile. Moving from one Forrest Tile to another does not renew the Armor. |
| ---           | ---                                                                                                                                                                  |
| **Plains**    | +1 Speed.                                                                                                                                                            |
| ---           | ---                                                                                                                                                                  |
| **Mountains** | Piece cannot be moved by an opponent's push, pull, or other forced movement skills.                                                                                  |
| ---           | ---                                                                                                                                                                  |

### **List of Skills:**

| **Category** | **Small/Big** | **Name**         | **Rune Cost** | **Description**                                                                                                    |
| ------------ | ------------- | ---------------- | ------------- | ------------------------------------------------------------------------------------------------------------------ |
| Strike       | Small         | Lance Thrust     | 2             | Target in Range -1 suffers 1 DMG.                                                                                  |
| ---          | ---           | ---              | ---           | ---                                                                                                                |
| Strike       | Small         | Hook Pull        | 3             | After the target suffers 1 DMG, pull it 1 tile closer along the skill path.                                        |
| ---          | ---           | ---              | ---           | ---                                                                                                                |
| Strike       | Small         | Armor Breaker    | 2             | Remove 1 Armor from the target (if any).                                                                           |
| ---          | ---           | ---              | ---           | ---                                                                                                                |
| Strike       | Small         | Rune Theft       | 3             | Target suffers 1 DMG. Steal 1 Rune from opponent (only if they have ≥1).                                           |
| ---          | ---           | ---              | ---           | ---                                                                                                                |
| Strike       | Big           | Blade Tempest    | 4             | Target suffers 1 DMG; adjacent tiles to the target take no damage but are pushed 1 tile back along the skill path. |
| ---          | ---           | ---              | ---           | ---                                                                                                                |
| Shield       | Small         | Rust Shield      | 2             | Self: Gain +1 Armor.                                                                                               |
| ---          | ---           | ---              | ---           | ---                                                                                                                |
| Shield       | Small         | Field Medic      | 3             | Heal Injured status from adjacent allies.                                                                          |
| ---          | ---           | ---              | ---           | ---                                                                                                                |
| Shield       | Small         | Armorsmith       | 3             | Give an adjacent ally +1 Armor.                                                                                    |
| ---          | ---           | ---              | ---           | ---                                                                                                                |
| Move         | Small         | Quick Dash       | 3             | Self: Move up to 2 tiles along the skill path.                                                                     |
| ---          | ---           | ---              | ---           | ---                                                                                                                |
| Move         | Small         | Air Blast        | 2             | Push an enemy target 1 tile away from the origin tile.                                                             |
| ---          | ---           | ---              | ---           | ---                                                                                                                |
| Move         | Small         | Precision Thrust | 3             | Push an enemy target 1 tile in any direction (attacker chooses).                                                   |
| ---          | ---           | ---              | ---           | ---                                                                                                                |
| Move         | Big           | Shadow Shift     | 4             | Swap your position with a chosen allied target.                                                                    |
| ---          | ---           | ---              | ---           | ---                                                                                                                |
| Move         | Big           | Retreat Plan     | 4             | Self: Follow skill path movement to one of your Guards (end adjacent to it) within Range +1.                       |
| ---          | ---           | ---              | ---           | ---                                                                                                                |
| Mystic       | Small         | Focus Strike     | 1             | Your next skill activation this turn gains +1 Range.                                                               |
| ---          | ---           | ---              | ---           | ---                                                                                                                |
| Mystic       | Small         | Blade Call       | 2             | One of your skills this turn may cost 1 extra Rune to deal +1 DMG (once).                                          |
| ---          | ---           | ---              | ---           | ---                                                                                                                |