//! A press on a selection group's bare space starts the selection at its nearest text.
use super::*;
use crate::{Cx, El, Env, TextStyle, col, el, text};

#[derive(Default)]
struct S { pressed: u32 }

fn view(_: &S, _: &mut Cx<S>) -> El {
    let style = TextStyle::new(14.0, 400.0).lh(20.0);
    let t = |content: &str, key: &str| text(content.to_owned()).key(key).font(style).w(200.0).selectable(true);
    col().key("root").size(400.0, 300.0)
        .child(col().key("group").w_full().padding(40.0).gap(20.0).selection_group(true)
            .child(t("alpha one", "a"))
            .child(t("beta two", "b"))
            .child(el().key("button").size(60.0, 20.0).on_press(|s: &mut S| s.pressed += 1)))
        .child(el().key("outside").size(400.0, 40.0))
}

fn runtime() -> Runtime<S> {
    let mut rt = Runtime::new(S::default(), view, (400.0, 300.0), Env::new(true, 1.0));
    rt.frame(0.0);
    rt
}

fn rect(rt: &Runtime<S>, key: &str) -> [f32; 4] {
    rt.elements().into_iter().find(|e| e.key.as_deref() == Some(key)).unwrap().rect
}

fn drag(rt: &mut Runtime<S>, from: (f32, f32), to: (f32, f32)) {
    rt.pointer_down(from.0, from.1);
    rt.frame(1.0 / 60.0);
    rt.pointer_move(to.0, to.1);
    rt.frame(1.0 / 60.0);
    rt.pointer_up(to.0, to.1);
    rt.frame(1.0 / 60.0);
}

#[test]
fn a_drag_from_the_margin_beside_a_text_selects_from_its_start() {
    let mut rt = runtime();
    let (a, b) = (rect(&rt, "a"), rect(&rt, "b"));
    // 20 points left of the first text, inside the group's padding; to past the second's end.
    drag(&mut rt, (a[0] - 20.0, a[1] + a[3] * 0.5), (b[0] + b[2] - 1.0, b[1] + b[3] * 0.5));
    assert_eq!(rt.selected_text().as_deref(), Some("alpha one\nbeta two"));
}

#[test]
fn a_drag_from_the_gap_between_texts_starts_at_the_nearer_one() {
    let mut rt = runtime();
    let (a, b) = (rect(&rt, "a"), rect(&rt, "b"));
    // 2 points under the end of the first text, in the gap; up to its start.
    drag(&mut rt, (a[0] + a[2] - 1.0, a[1] + a[3] + 2.0), (a[0], a[1] + a[3] * 0.5));
    assert_eq!(rt.selected_text().as_deref(), Some("alpha one"));
    // 2 points above the second text: it starts there instead.
    drag(&mut rt, (b[0] + b[2] - 1.0, b[1] - 2.0), (b[0], b[1] + b[3] * 0.5));
    assert_eq!(rt.selected_text().as_deref(), Some("beta two"));
}

#[test]
fn a_press_on_a_control_in_the_group_or_outside_it_selects_nothing() {
    let mut rt = runtime();
    let (a, button, outside) = (rect(&rt, "a"), rect(&rt, "button"), rect(&rt, "outside"));
    drag(&mut rt, (button[0] + 5.0, button[1] + 5.0), (a[0] + a[2], a[1] + 5.0));
    assert_eq!(rt.selected_text(), None);
    assert_eq!(rt.state.pressed, 0, "a drag off the button is no press");
    drag(&mut rt, (outside[0] + 5.0, outside[1] + 5.0), (a[0] + a[2], a[1] + 5.0));
    assert_eq!(rt.selected_text(), None);
}
