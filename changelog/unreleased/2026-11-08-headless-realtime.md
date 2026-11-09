- **`--realtime` paces a headless run by the wall clock.** A headless run draws its frames as fast as
  it can, so a script's "wait two seconds" passes in a few milliseconds. Against a live server or a
  process outside, the answer then lands at an arbitrary step. With `--realtime`, each frame waits
  until its time comes, so scripted steps and outside events meet on the times the script names.
- **`--trees DIR` writes the layout tree at every script mark.** `--tree` writes one tree, at `--at`,
  so checking a journey after each of its steps took one run per step. With `--trees`, each mark a
  script passes leaves `DIR/<mark>.json`, drawn in the frame the mark falls in.
