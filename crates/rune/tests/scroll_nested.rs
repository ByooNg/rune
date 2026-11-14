//! A vertical wheel or trackpad gesture inside nested vertical viewports: which viewport moves.
//! A page holds a short box with its own vertical scroll that has nothing to scroll (a capped
//! code block that fits), and a log that does overflow. A gesture moves the innermost viewport
//! under the pointer that can move its way when it begins, and keeps moving that one, as AppKit
//! latches a scroll gesture to the view it began on.
use rune::{Cx, El, Env, GesturePhase, Runtime, ScrollSpec, WheelEvent, col, el};

#[derive(Default)]
struct S { prepended: usize }

const ROW_H: f32 = 40.0;

fn rows(prefix: &str, n: usize) -> El {
    (0..n).fold(col().w_full(), |c, i| c.child(el().key(format!("{prefix}{i}")).w_full().h(ROW_H)))
}

/// A 300 pt page: 10 rows, a 50 pt box holding 30 pt (nothing to scroll), 10 rows, a 50 pt log
/// holding 200 pt, 20 rows. `prepended` rows go above everything, as older messages load.
fn view(s: &S, _: &mut Cx<S>) -> El {
    let fits = col().key("fits").w_full().h(50.0).scroll(ScrollSpec { stick_bottom: false }).child(el().w_full().h(30.0));
    let log = col().key("log").w_full().h(50.0).scroll(ScrollSpec { stick_bottom: false }).child(rows("line", 5));
    let content = col().w_full()
        .child(rows("old", s.prepended))
        .child(rows("a", 10)).child(fits).child(rows("b", 10)).child(log).child(rows("c", 20));
    col().w_full().h_full().child(col().key("page").w_full().h(300.0).scroll(ScrollSpec { stick_bottom: false }).child(content))
}

fn app() -> Runtime<S> {
    let mut rt = Runtime::new(S::default(), view, (300.0, 300.0), Env::new(true, 1.0));
    rt.frame(0.0);
    rt
}

fn y_of(rt: &Runtime<S>, key: &str) -> f32 {
    rt.elements().into_iter().find(|e| e.key.as_deref() == Some(key)).unwrap_or_else(|| panic!("{key}")).rect[1]
}

/// One trackpad gesture of `deltas` deltas of `dy` at (150, `y`), a frame after each.
fn swipe(rt: &mut Runtime<S>, y: f32, dy: f32, deltas: usize) {
    for k in 0..deltas {
        let phase = if k == 0 { GesturePhase::Began } else { GesturePhase::Moved };
        rt.wheel_event(WheelEvent::new(0.0, dy).at(150.0, y).phase(phase).precise(true));
        rt.frame(1.0 / 60.0);
    }
    rt.wheel_event(WheelEvent::new(0.0, 0.0).at(150.0, y).phase(GesturePhase::Ended).precise(true));
    rt.frame(0.15);
}

#[test]
fn a_viewport_with_nothing_to_scroll_passes_the_wheel_to_the_page() {
    let mut rt = app();
    // "fits" spans 400..450 in the page's content: scroll it under the pointer at y 150.
    swipe(&mut rt, 150.0, -10.0, 26);
    let fits = y_of(&rt, "fits");
    assert!(fits <= 150.0 && fits + 50.0 > 150.0, "the box is under the pointer: {fits}");
    let page_row = y_of(&rt, "b0");
    swipe(&mut rt, 150.0, -10.0, 5);
    assert_eq!(y_of(&rt, "b0"), page_row - 50.0, "a gesture begun over the box scrolls the page");
}

#[test]
fn a_gesture_keeps_its_viewport_when_another_slides_under_the_pointer() {
    let mut rt = app();
    // One gesture of 60 deltas of 10 pt: "fits" (400) and then "log" (850) pass under y 150.
    let start = y_of(&rt, "c0");
    swipe(&mut rt, 150.0, -10.0, 60);
    assert_eq!(y_of(&rt, "c0"), start - 600.0, "every delta moved the page");
    assert_eq!(y_of(&rt, "line0") - y_of(&rt, "log"), 0.0, "the log never moved");
}

#[test]
fn a_gesture_begun_over_a_viewport_that_can_move_scrolls_it() {
    let mut rt = app();
    swipe(&mut rt, 150.0, -10.0, 70);
    let log = y_of(&rt, "log");
    assert!(log <= 150.0 && log + 50.0 > 150.0, "the log is under the pointer: {log}");
    let page_row = y_of(&rt, "c0");
    swipe(&mut rt, 150.0, -10.0, 5);
    assert_eq!(y_of(&rt, "line0") - y_of(&rt, "log"), -50.0, "the log moved");
    assert_eq!(y_of(&rt, "c0"), page_row, "the page held");
}

#[test]
fn rows_kept_on_screen_keep_the_wheel_delta_that_came_before_the_commit() {
    let mut rt = app();
    swipe(&mut rt, 30.0, -10.0, 20);
    let before = y_of(&rt, "b0");
    // A delta lands between frames, then older rows load above in the same frame.
    rt.wheel_event(WheelEvent::new(0.0, 25.0).at(150.0, 30.0).phase(GesturePhase::Began).precise(true));
    rt.update(|s, tx| {
        s.prepended = 5;
        tx.instant();
        tx.keep_on_screen("b0");
    });
    rt.frame(1.0 / 60.0);
    assert_eq!(y_of(&rt, "b0"), before + 25.0, "the row stays where the wheel put it");
}
