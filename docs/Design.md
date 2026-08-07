# Tiny Weather Office — Design

## Product vision

Tiny Weather Office is a relaxing portrait-mobile logic puzzle. The player operates a miniature, overly serious weather bureau by placing rain machines on a colored climate map. The puzzle should be understandable at a glance, satisfying on every move, and progressively harder without accumulating exceptions.

The personality comes from the contrast between bureaucratic presentation and tiny tactile weather machines. The logic must remain more prominent than the decoration.

## Design principles

1. **Rules remain visible.** The four placement rules are always shown during play; the player never needs to reopen a tutorial.
2. **The board is the hero.** Climate regions, flowers, machines, and X marks must be distinguishable within a second.
3. **Every action has one meaning.** Tap and drag create X marks; double-tap attempts a rain-machine placement.
4. **Feedback is calm and local.** Validated machines receive a short green pulse, nearby flowers bloom, and mistakes consume one clearly displayed life without interrupting play while lives remain.
5. **Difficulty comes from deduction.** Board size, region geometry, and flower placement create harder levels while the interaction vocabulary stays fixed.

## Primary gameplay screen

The game is designed for a `390 × 844` portrait viewport.

### Information hierarchy

From top to bottom:

1. Back button, level title, and settings.
2. Placement progress, such as `3/6 placed`, and remaining lives.
3. A compact `2 × 2` rules block:
   - `1 per climate`
   - `1 per row & column`
   - `Machines don't touch`
   - `Flowers touch rain`
4. The climate grid, occupying most of the screen.
5. A persistent gesture legend:
   - `Tap or drag: X`
   - `Double-tap: place & check`
6. Hint.

The layout must remove decoration before reducing board size or text legibility.

## Visual language

### Art direction

- Tactile clay-diorama objects with crisp, flat interaction overlays.
- Warm paper-cream background.
- Pastel climate regions: mist blue, moss, ochre, coral, lavender, and muted teal.
- Dark navy typography and X marks.
- Green reserved for a newly validated machine.
- Wilted flowers use muted tan; watered flowers use fresh pink or yellow.

The tone is calm, playful, and slightly absurd, but not childish.

### Board states

| Element | Visual treatment |
| --- | --- |
| Empty cell | Region color and subtle clay texture |
| X mark | Large, centered, dark-navy X |
| Mistake X | Large, centered, red X that cannot be removed |
| Rain machine | High-contrast miniature machine centered in the cell |
| Newly validated machine | Brief green ring, check pulse, and water droplets |
| Dry flower | Tan, drooping, visibly wilted |
| Watered flower | Upright, saturated, visibly blooming |
| Climate boundary | Stronger than normal cell boundaries, but still soft |

Flowers are fixed clues, not player pieces. Their cells cannot receive X marks or machines.

## Interaction design

### Marking impossible cells

- A single tap toggles an X on an eligible empty cell.
- Pressing and dragging enters X-paint mode.
- The drag may cross rows, columns, or an arbitrary path.
- Re-entering the same cell during one drag does not toggle it repeatedly.
- Starting a drag on an X erases X marks along the stroke; starting on an empty cell adds them.
- Dragging skips flowers, validated machines, and red mistake X marks.
- X marks are neutral notes and are never checked for correctness.

### Placing a rain machine

- A double-tap on an empty cell attempts to place a machine.
- To place on a player-marked cell, the player must first tap once to remove the X, then double-tap the empty cell.
- Double-tapping any X is ignored.
- The pending single-tap action must be cancelled, so the first tap does not flash an X.
- A correct placement is accepted and receives a short green validation animation.
- An accepted machine cannot be removed.
- An incorrect placement loses one life and replaces the cell with a permanent red X, accompanied by a restrained outline and soft horizontal shake.
- Losing the final life ends the attempt.

### Flower feedback

Whenever a validated machine is added, all dry flowers in its eight-cell neighborhood bloom. Flower state is derived from machine placement rather than stored independently.

## Motion and sound

- X placement: immediate, quiet scale-in.
- Drag marking: light rhythmic ticks, rate-limited during fast movement.
- Correct machine: short mechanical click, green pulse, water droplets, then flower bloom.
- Incorrect machine: permanent red X, one-life decrement, soft wooden knock, and a small shake; avoid alarm sounds.
- Level completion: climate regions brighten in sequence, followed by one concise weather-office stamp.

Animations should clarify state changes and complete quickly. No animation should block the next move.

## Accessibility

- Do not rely on color alone: every climate has both a color and a subtle texture.
- Maintain at least `4.5:1` contrast for text and interaction marks.
- Touch targets are at least `44 × 44` points.
- Provide reduced-motion and haptics toggles.
- Distinguish dry and watered flowers through silhouette as well as color.
- All rules and gestures must have text labels in addition to icons.

## Difficulty progression

| Stage | Grid | Changes |
| --- | ---: | --- |
| Introduction | `4 × 4` | One machine per climate, row, and column |
| Separation | `4 × 4` | Introduce the no-touch rule |
| First flowers | `5 × 5` | Add a small number of flower clues |
| Shared coverage | `5 × 5` | One machine may water multiple flowers |
| Full forecast | `6 × 6+` | Irregular regions and overlapping flower constraints |

New mechanics are introduced individually, then combined. The final rules remain unchanged after the full forecast stage.

## Out of scope for the core game

- Timers, currencies, shops, and leaderboards.
- Multiple machine types or machine-specific abilities.
- Hidden gesture-only features.
- Validating X marks.
- Decorative elements that obscure cells, regions, or clues.
