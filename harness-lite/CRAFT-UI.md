# Harness Lite — screen craft floor

Read this when you build, change or review a screen. It extends the [shared contract](CONTRACT.md). Written for phone apps (iOS first); on the web, apply the lines that make sense and the web numbers given.

The floor is counted per **component type**, not per screen: an app has a dozen or so types (sheet, text field, primary button, list …) however many screens it has. Fix a type once, where it is defined, and every screen that uses it is fixed. A screen is done when every type on it meets its lines and the journey it serves works.

A floor catches slips and things forgotten. It does not catch a screen that solves the wrong problem; that is what the blueprint's journeys and Người soát are for. A line that does not fit the product is skipped with the reason written down; that is better than following it in the wrong place.

## The lines

**Any screen reached from another**
- There is a visible way back or a visible way to close, and the system gesture for it works (edge swipe on a pushed screen). A gesture is never the only way: a button does the same.
- Going back does not silently throw away what the user typed.
- Content and controls stay inside the safe area: nothing sits under the notch, the home indicator, a tab bar or another layer.

**Sheet or modal**
- It closes by a clearly labelled button; a sheet also closes by pulling down. If closing would lose input, it asks first.
- A modal that needs an answer does not close on a tap outside.

**Popover, menu, picker (a small layer that does not block the screen)**
- A tap outside closes it, and the work underneath is kept.

**Text field and keyboard**
- The keyboard can be dismissed in an obvious way, and it goes away after the user submits.
- With the keyboard open, the field being edited and the main action stay visible and tappable.
- The keyboard type matches the data (number, email, URL).
- Input is checked as it is entered where possible, and an error says which field and what would be accepted.

**Button and any tappable control**
- The touch target is at least 44×44 pt on iOS (web: 24×24 CSS px at the very least), and neighbouring targets have space between them; spacing matters as much as size.
- Pressing it gives a visible response within about a tenth of a second.
- Anything longer shows that work is in progress, says how long when it can, and cannot be fired twice.
- The main action of the screen is reachable with the thumb of the hand holding the phone.

**Destructive or irreversible action**
- Undo is offered where it is possible. Where it is not, the user is warned before, in words that say what will be lost.

**List and any view of data**
- It has four states: loading, empty, failed, filled. The empty and failed states say what the user can do next.
- It is built and checked against data from the real source, not only samples written to fit.

**Error message**
- Plain words, the exact thing that went wrong, and what to do next. No codes alone.

**Text and colour**
- Body text has contrast of at least 4.5:1 against its background, interface parts and graphics at least 3:1, in light and dark. Measure on the colour values; do not judge by eye.
- Nothing is cut off or overlaps at the largest system text size, in dark mode, or when the device rotates if rotation is allowed.

**Motion, haptics, time**
- Motion is reduced when the user has asked the system for that.
- A haptic is used only with the meaning the system gives it, sparingly, and never as the only signal.
- Nothing important disappears on a timer the user cannot extend.

**Permission prompt**
- It is asked at the moment the user reaches for the feature that needs it, not at launch.

**Across the app**
- The same thing looks and behaves the same everywhere, and follows what the platform's own apps do, so the user recognises instead of having to remember.

## Checking and reporting

Check by operating the component (simulator or device: tap it, open the keyboard, go back, enlarge the text), not by reading the code, except where a line says to measure. In the hand-back or review, list the component types touched and for each line: met (with the screenshot or test), not applicable (with the reason), or not checked (with why). "Probably fine" is not checked.

Whether it is beautiful, in the right taste, or worth using stays with Chủ nhà.

## How this file changes

A basic screen defect that reaches Chủ nhà becomes a line here, under its component type. A line that every app now enforces by an automatic test is removed.
