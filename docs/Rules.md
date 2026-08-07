# Tiny Weather Office — Rules

## Objective

Place all rain machines so that every climate receives one machine and every dry flower is watered.

## Placement rules

1. **One per climate**  
   Every colored climate region contains exactly one rain machine.

2. **One per row and column**  
   Every row contains exactly one rain machine, and every column contains exactly one rain machine.

3. **Machines cannot touch**  
   Two rain machines cannot occupy neighboring cells—not horizontally, vertically, or diagonally.

4. **Every flower touches rain**  
   Every flower must have a rain machine in one of the eight cells immediately around it: above, below, left, right, or diagonally.

Flower cells are fixed clues. You cannot place a machine or an X on them.

## Controls

- **Tap an eligible cell:** place or remove your X.
- **Drag across cells:** place X marks continuously, or erase them when the drag starts on an X.
- **Double-tap an empty cell:** attempt to place a rain machine and check that placement.
- **Hint:** reveal one logically justified next step.

Your X marks are removable personal notes. The game never checks whether they are correct.
Red X marks record incorrect machine attempts and cannot be removed.

## Validation

When you double-tap:

- A correct machine remains on the board and briefly glows green.
- A correct machine cannot be removed.
- An incorrect machine costs one life and leaves a permanent red X on that cell.

To place a machine on a cell with your X, tap once to remove the X, then double-tap the empty cell. Double-tapping an X does nothing.

Flowers around an accepted machine bloom immediately. A flower remains dry until at least one accepted machine is next to it.

## Winning

You win when:

- all required rain machines are placed;
- every climate has exactly one machine;
- every row and column has exactly one machine;
- no machines touch;
- every flower is watered.

You lose when no lives remain. Until then, you can revise your own X marks and continue until the forecast is complete.
