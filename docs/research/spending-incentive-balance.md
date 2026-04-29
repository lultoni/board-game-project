<img src="https://r2cdn.perplexity.ai/pplx-full-logo-primary-dark%402x.png" style="height:64px;margin-right:32px"/>

# RESEARCH REQUEST: How do resource-scarce tactical games balance "abilities should feel costly" vs "not using them should feel costly"?

Context: We are designing a 2-player perfect-information tactical board game where players spend Runes (currency) to activate skills. Income scales over the game: early game (~2/turn, skills cost 2-4) creates hard scarcity; late game
(~4-5/turn) creates opportunity richness where players can afford multiple skills but never everything they want. We need a design principle that tells us HOW to make spending feel meaningful across this arc without either (a) hoarding
being optimal or (b) spending feeling free/automatic.

Questions:

1. In games with scaling resource income (e.g., Twilight Imperium influence, Spirit Island energy, Race for the Galaxy cards-as-currency, Res Arcana essences), how do designers keep spending decisions MEANINGFUL in the late game when
income is high? What prevents "just spend everything every turn" from being correct?
2. How do tactical games create "forced spending" (you MUST respond or lose something) without making the spender feel helpless? What's the difference between "I chose to spend because the threat demanded it" (good) vs "I was taxed" (bad,
feels punishing)?
3. Is there a known design principle or framework for the TRANSITION from scarcity-driven spending (early: "can I afford this?") to opportunity-cost-driven spending (late: "which of these good options do I pick?")? How do designers ensure
both phases feel strategic rather than one feeling like "waiting" and the other like "spending without thinking"?
4. What role does TEMPO play in spending decisions — specifically, how do games reward spending NOW vs saving for a bigger play later? Examples from Chess (tempo moves), Go (sente/gote valuation), BattleCon (ante system), Netrunner (click
efficiency)?

Please include:

- Specific mechanics from published games, not just philosophy
- Designer commentary or postmortems where available
- Any examples of games that FAILED at this balance (spending felt trivial or hoarding dominated) and what went wrong
- Concrete principles/heuristics that emerged from these designs

To balance resource scarcity with late-game opportunity richness, tactical games use exponential option growth and strict action limits to ensure spending decisions remain agonizing. By shifting the primary constraint from pure currency early on to action economy, timing, and spatial positioning later, designers prevent both passive hoarding and mindless spending.

## Late-Game Meaningful Spending

In games with scaling economies, designers prevent automatic spending by ensuring the volume of viable options grows exponentially faster than linear income. In *Race for the Galaxy*, cards serve as both actions and currency, forcing an agonizing opportunity cost where spending a card to build an engine means permanently sacrificing the ability to play that card. *Spirit Island* pairs high late-game energy generation with complex board states and escalating invader threats, shifting the puzzle from affordability to the spatial and temporal consequences of your powers. *Twilight Imperium* achieves this by using Command Tokens for both tactical movement and secondary strategy actions, ensuring that even with high late-game influence, committing a token to one system leaves you vulnerable elsewhere. Without action caps or hand limits to restrict output, late-game wealth simply breaks the game; for example, early versions of *Artifact* suffered because an uncapped resource system heavily rewarded raw card power over tactical restraint.[^1][^2][^3][^4][^5][^6]

## Forced Spending vs Taxation

"Forced spending" feels strategic when it empowers the player to overcome a dynamic puzzle, whereas a "tax" feels punishing because it removes agency. In *Netrunner*, the corporation installs ICE that forces the runner to spend credits to bypass it. This feels like an active, strategic choice because the runner decides when and where to initiate the run, turning the cost into a tactical optimization puzzle. Conversely, unconditional upkeep costs or flat degradation mechanics often feel like taxes because they demand payment merely to maintain the status quo without advancing the player's board state. A successful design principle is to tie mandatory spending to active threat mitigation, allowing players to feel clever about how they respond rather than annoyed that they must respond.[^7]

## Scarcity to Opportunity Transition

The transition from early-game scarcity to late-game richness relies on the principle of the "action bottleneck." In the early game, the player's primary constraint is currency, but as income scales, the game must introduce a hard cap on how many actions can be performed per turn. In *Res Arcana*, players can generate massive amounts of essence in the late game, but the race to 10 victory points forces them to optimize for hyper-efficient, single-turn conversions before their opponent wins. This design ensures that early-game spending is about building an engine, while late-game spending is about finding the optimal permutation of synergistic abilities within a strict action or time limit. If a game fails to introduce a secondary bottleneck like a strict turn limit or an action cap, the late game devolves into a repetitive exercise of clicking every available button.[^5][^8]

## Tempo and Timing Decisions

Tempo is the advantage gained by dictating the pace of the match, forcing the opponent to react rather than execute their own strategy. Games reward immediate spending when it seizes the initiative, but must balance this with the threat of overextending.


| Game | Tempo Mechanic | Spending vs Saving Dynamic |
| :-- | :-- | :-- |
| **Chess** | Initiative | Moving a piece to threaten the opponent forces them to spend their turn reacting, granting the attacker tempo. |
| **BattleCON** | Ante System | Players secretly bid Overload tokens to increase their attack's priority; spending now secures the first strike to stun the opponent and negate their attack [^9][^10]. |
| **Netrunner** | Click Efficiency | Players have four actions (clicks) per turn; saving credits costs valuable clicks that could otherwise be used to apply immediate pressure to the opponent [^11][^12]. |

To ensure hoarding doesn't become mathematically optimal, the value of saved Runes must degrade over time compared to the immediate board presence gained by spending them. Are unspent Runes carried over between turns in your game, or do players lose them at the end of the round?
<span style="display:none">[^13][^14][^15][^16][^17][^18][^19][^20][^21][^22][^23][^24][^25][^26][^27][^28][^29][^30][^31][^32][^33][^34][^35][^36][^37][^38][^39][^40][^41][^42][^43][^44][^45][^46][^47][^48][^49][^50]</span>

<div align="center">⁂</div>

[^1]: https://en.wikipedia.org/wiki/List_of_commercial_failures_in_video_and_arcade_games

[^2]: https://spirit-island-jagged-earth.backerkit.com/hosted_preorders/project_updates?page=7

[^3]: https://www.shutupandsitdown.com/review-spirit-island/

[^4]: https://www.3spellcastersandadwarf.com/gaming-blog/race-for-the-galaxy

[^5]: https://daoofboardgaming.home.blog/2021/08/12/race-for-the-galaxy/

[^6]: https://twilight-imperium.fandom.com/wiki/Command_Tokens

[^7]: https://www.reddit.com/r/Netrunner/comments/1ge9a5p/i_want_peoples_opinions_on_costbalancing_in/

[^8]: https://opinionatedgamers.com/2020/01/08/dale-yu-first-impressions-of-res-arcana-lux-et-tenebrae/

[^9]: https://www.youtube.com/watch?v=yiL1cF9aGSQ

[^10]: https://gamers-hq.de/media/pdf/73/5b/2e/BattleCON-Web-Rulebook-v4-Single-Pages-version.pdf

[^11]: https://rules.nullsignal.games

[^12]: https://rubenpieters.github.io/netrunner-comprehensive-rules/

[^13]: https://www.reddit.com/r/RPGdesign/comments/9vasa7/how_do_you_balance_resource_costs_for_spells_and/

[^14]: https://gamebalanceconcepts.wordpress.com/2010/07/21/level-3-transitive-mechanics-and-cost-curves/

[^15]: https://waywardstrategy.com/2018/03/28/the-cost-of-combat-in-strategy-games/

[^16]: https://www.youtube.com/watch?v=N5IjVDpTJT0

[^17]: https://chillplacegaming.com/resource-management-tactics/

[^18]: https://www.gamegrin.com/articles/why-games-fail/

[^19]: https://www.ttrpg-games.com/blog/how-to-design-balanced-resource-systems-in-rpgs/

[^20]: https://www.facebook.com/groups/558977328068750/posts/1545262432773563/

[^21]: https://www.binance.com/en/square/post/661180

[^22]: https://www.linkedin.com/advice/0/how-can-you-balance-resource-management-make-game-challenging-97umf

[^23]: https://www.watchmojo.com/articles/top-10-big-budget-aaa-games-that-failed-miserably

[^24]: https://onlinelibrary.wiley.com/doi/10.1155/2019/5475341

[^25]: https://arxiv.org/pdf/2402.04328.pdf

[^26]: https://arxiv.org/pdf/2401.04264.pdf

[^27]: http://arxiv.org/pdf/2401.14613.pdf

[^28]: http://arxiv.org/pdf/2301.08830.pdf

[^29]: https://arxiv.org/pdf/2503.24099.pdf

[^30]: http://arxiv.org/pdf/2502.07652.pdf

[^31]: http://arxiv.org/pdf/2110.12099.pdf

[^32]: http://link.springer.com/10.1007/978-3-642-18948-7_20

[^33]: https://rmets.onlinelibrary.wiley.com/doi/10.1002/wea.542

[^34]: https://www.semanticscholar.org/paper/4e02e505574425d24502b62116749161c7ef23a9

[^35]: https://www.semanticscholar.org/paper/8bc64ddd84f5ce0da8af03f4ebccb5dc95d89f18

[^36]: https://www.semanticscholar.org/paper/6194f9e50f51edbb25b4681b89e7c48a8e17ea70

[^37]: https://morepress.unizd.hr/journals/index.php/arsadriatica/article/view/520

[^38]: https://www.semanticscholar.org/paper/846689c820cea144c7b949700c5bab5110ea3956

[^39]: http://www.tandfonline.com/doi/abs/10.1080/02690051003651472

[^40]: https://www.reddit.com/r/spiritisland/comments/zpyzd7/help_me_balance_my_custom_spirit_designed_for_my/

[^41]: https://www.facebook.com/groups/soloboardgamers/posts/2761134744220979/

[^42]: https://www.instagram.com/p/DUBZi8SiG0b/

[^43]: https://tvtropes.org/pmwiki/pmwiki.php/Characters/BattleCon

[^44]: https://www.facebook.com/groups/GameDesignersGuild/posts/4043550772412304/

[^45]: https://www.reddit.com/r/twilightimperium/comments/11r4xbd/whats_the_best_strategy_for_playing_the_arborec/

[^46]: https://www.facebook.com/groups/boardgamerevolution/posts/2694848364140418/

[^47]: https://litko.net/products/litko-twilight-imperium-command-control-tokens

[^48]: https://cdn.1j1ju.com/medias/ad/a7/0b-twilight-imperium-quatrieme-edition-rulebook.pdf

[^49]: https://en.wikipedia.org/wiki/Twilight_Imperium

[^50]: https://www.reddit.com/r/boardgames/comments/lo0x6b/res_arcana_does_it_have_staying_power/

