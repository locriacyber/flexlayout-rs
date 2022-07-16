This is a library for typesetting axis-aligned rectangles in 2-Euclidean. I only layout rectangles according to flex box model.

This is a Rust translation of https://github.com/randrew/layout.

***
I made this library because I want to use flex box layout without bundled with existing ui library.

You should use this library like this:

1. Layout (get position and size (rectangle) of ui widgets)
2a. Trigger input events for UI
2b. Draw UI

Since layout is defined before event handling (unlike imgui), every frame can be perfect.

## TODO

- [ ] This algorithm can be easily extended to other euclidean geometry. Maybe it's useful?