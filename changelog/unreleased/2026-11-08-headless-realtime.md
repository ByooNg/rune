- **`--realtime` paces a headless run by the wall clock.** A headless run draws its frames as fast as
  it can, so a script's "wait two seconds" passes in a few milliseconds. Against a live server or a
  process outside, the answer then lands at an arbitrary step. With `--realtime`, each frame waits
  until its time comes, so scripted steps and outside events meet on the times the script names.
