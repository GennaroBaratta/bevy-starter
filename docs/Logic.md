# Tiny Weather Office — Logic

## Scope

This document defines the deterministic game model, gesture interpretation, validation, win and loss detection, puzzle generation, and minimum test contract. Presentation details belong in `Design.md`; player-facing wording belongs in `Rules.md`.

## Coordinate model

Use zero-based coordinates:

```ts
type Cell = {
  row: number;
  column: number;
};
```

For an `N × N` board, valid coordinates satisfy:

```text
0 <= row < N
0 <= column < N
```

Each cell belongs to exactly one climate region. A flower may occupy a cell as a fixed clue; flower cells are not eligible for player marks or machines.

## Puzzle definition

```ts
type CellKey = `${number}:${number}`;
type RegionId = string;

type PuzzleDefinition = {
  id: string;
  size: number;
  regionByCell: Record<CellKey, RegionId>;
  flowers: ReadonlySet<CellKey>;
  solutionMachines: ReadonlySet<CellKey>;
  startingLives: number;
};
```

Required definition invariants:

- The board is square.
- Every board cell has exactly one region.
- The number of regions equals `size`.
- Every region is orthogonally connected.
- No flower cell is a solution-machine cell.
- `solutionMachines` satisfies every rule and contains exactly `size` cells.
- The puzzle has exactly one valid solution.
- `startingLives` is a positive integer.

## Runtime state

```ts
type GameState = {
  playerXMarks: Set<CellKey>;
  mistakeXMarks: Set<CellKey>;
  machines: Set<CellKey>;
  livesRemaining: number;
  status: "playing" | "complete" | "lost";
};
```

`livesRemaining` starts at `puzzle.startingLives`. `wateredFlowers` is derived from `machines`; it is not stored. Transient animation state, such as the most recently validated machine, belongs to the view layer rather than `GameState`. Accepted machines and mistake X marks are permanent for the attempt. Gameplay input is ignored unless the status is `"playing"`.

## Neighborhoods

### Eight-cell adjacency

Machines touching and flowers receiving water both use Chebyshev adjacency:

```ts
function touches(a: Cell, b: Cell): boolean {
  const dr = Math.abs(a.row - b.row);
  const dc = Math.abs(a.column - b.column);
  return Math.max(dr, dc) === 1;
}
```

This includes horizontal, vertical, and diagonal neighbors, but excludes the same cell.

### Flower coverage

```ts
function isFlowerWatered(
  flower: Cell,
  machines: Iterable<Cell>,
): boolean {
  return Array.from(machines).some((machine) => touches(flower, machine));
}
```

## Rule predicates

For a complete candidate machine set `M`:

```ts
function hasOnePerRow(M: Cell[], size: number): boolean;
function hasOnePerColumn(M: Cell[], size: number): boolean;
function hasOnePerRegion(M: Cell[], puzzle: PuzzleDefinition): boolean;
function hasNoTouchingMachines(M: Cell[]): boolean;
function watersEveryFlower(M: Cell[], flowers: Cell[]): boolean;
```

Their semantics are:

- `|M| = N`.
- Every row count is exactly `1`.
- Every column count is exactly `1`.
- Every region count is exactly `1`.
- No pair of machines has Chebyshev distance `1`.
- Every flower has at least one adjacent machine.

## X-mark interaction

### Single tap

A single tap toggles an X only when the cell:

- is inside the board;
- is not a flower;
- does not contain a validated machine;
- does not contain a mistake X.

X correctness is never evaluated.

### Drag painting

On pointer down and movement beyond the drag threshold:

1. Cancel any pending single-tap timer.
2. Enter X-paint mode.
3. Choose the stroke operation from the first eligible cell:
   - empty first cell → add X marks;
   - marked first cell → erase X marks.
4. Apply the same operation to each newly entered eligible cell.
5. Ignore repeated visits to the same cell during the stroke.

This supports rows, columns, and arbitrary paths without toggling cells unpredictably. Flowers, machines, and mistake X marks are never eligible.

## Double-tap arbitration

Single tap and double-tap share the same surface, so the recognizer must defer committing the X until the double-tap window closes.

Recommended defaults:

```ts
const DOUBLE_TAP_WINDOW_MS = 250;
const DOUBLE_TAP_DISTANCE_PX = 24;
const DRAG_THRESHOLD_PX = 8;
```

The values are configuration inputs for usability tuning, not game rules.

On a recognized double-tap:

1. Cancel the pending single-tap X action.
2. Ignore the request unless the cell is empty. Player X marks, mistake X marks, flowers, and machines are not empty.
3. Validate the attempted cell.
4. If accepted, add a permanent machine.
5. If rejected, add a permanent mistake X and subtract one life.
6. If no lives remain, set the status to `"lost"`.

## Machine validation

The specified interaction validates each machine against the canonical solution:

```ts
function validateMachineAttempt(
  cell: CellKey,
  puzzle: PuzzleDefinition,
): "accepted" | "rejected" {
  return puzzle.solutionMachines.has(cell) ? "accepted" : "rejected";
}
```

An accepted placement triggers view feedback and recomputes flower state. A rejected placement changes only the attempted cell, the remaining life count, and possibly the game status; it does not affect player X marks elsewhere.

### Product consequence

Because incorrect cells can be tested without a penalty, canonical-solution validation permits brute-force probing. This is consistent with a forgiving game, but it weakens deduction. If playtesting shows excessive probing, keep the same visuals and change validation to reject only visible rule conflicts during play, then validate the full solution at completion.

## Derived flower state

```ts
function getWateredFlowers(
  puzzle: PuzzleDefinition,
  machines: Set<CellKey>,
): Set<CellKey>;
```

For each flower, convert its key to a coordinate and check whether any current machine `touches` it. A flower blooms when the predicate changes from false to true. Because machines cannot be removed, watered flowers do not become dry again during an attempt.

## Win condition

```ts
function isComplete(
  puzzle: PuzzleDefinition,
  state: GameState,
): boolean {
  const machines = [...state.machines].map(parseCellKey);

  return (
    machines.length === puzzle.size &&
    hasOnePerRow(machines, puzzle.size) &&
    hasOnePerColumn(machines, puzzle.size) &&
    hasOnePerRegion(machines, puzzle) &&
    hasNoTouchingMachines(machines) &&
    watersEveryFlower(machines, [...puzzle.flowers].map(parseCellKey))
  );
}
```

Completion is checked after every accepted machine placement. Neither kind of X mark participates. A completed game cannot subsequently become lost.

## Hint logic

A hint must reveal a deduction, not an arbitrary solution cell.

Preferred order:

1. A cell invalidated by an already placed machine's row or column.
2. A cell invalidated by machine-touch adjacency.
3. A climate reduced to one remaining candidate.
4. A row or column reduced to one remaining candidate.
5. A flower whose remaining coverage candidates force a machine position.

The hint may add one X or highlight one forced machine cell. It should also state the rule that justified the deduction.

## Puzzle generation

Generate and verify puzzles offline or at build time:

1. Select `N` machine cells forming a row/column permutation.
2. Reject permutations where machines in consecutive rows occupy adjacent columns; this enforces diagonal separation.
3. Partition the board into `N` orthogonally connected climate regions, each containing exactly one solution machine.
4. Add flower clues only to non-machine cells adjacent to at least one solution machine.
5. Run the solver and retain only puzzles with exactly one solution.
6. Rate difficulty using solver deductions and branching, not only board size.

### Solver outline

Use constraint propagation followed by backtracking:

- Candidate variable: whether each eligible cell contains a machine.
- Exact-one constraints: every row, column, and climate region.
- At-most-one constraints: all touching cell pairs.
- At-least-one constraints: the adjacent candidates around every flower.
- Stop search after finding two solutions; only zero, one, or multiple matters for generation.

## Minimum test contract

1. Every shipped puzzle passes all definition invariants and has one solution.
2. Row, column, region, touching, and flower predicates are unit-tested independently.
3. Diagonal machine adjacency is rejected.
4. Diagonal flower coverage is accepted.
5. Flower cells reject both X marks and machines.
6. A double-tap is ignored unless the target cell was empty before the gesture; placing on a player X requires removing it first.
7. A drag stroke is idempotent per visited cell and can add or erase player X marks.
8. A rejected machine attempt adds a permanent mistake X and subtracts exactly one life.
9. Player input cannot remove machines or mistake X marks.
10. Reaching zero lives changes the status to `"lost"`.
11. Completion ignores both kinds of X mark and requires every machine and flower constraint.
