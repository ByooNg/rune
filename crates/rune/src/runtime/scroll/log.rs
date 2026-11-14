//! `RUNE_SCROLL_LOG=<file>`: a JSON line for each frame of every scroll viewport keyed
//! `RUNE_SCROLL_LOG_KEY` (default `transcript`), with its offset and the rows it shows, and one
//! for each wheel delta with the viewport it moved. A check driving a real window reads it to
//! tell a jump (a row moving other than the wheel asked) from a scroll, and a delta a nested
//! viewport swallowed from one the page took. Off, each frame checks one cached flag.
//! Times are Unix milliseconds, so a log the app writes lines up with this one.
use super::*;
use std::cell::RefCell;
use std::io::Write;

/// Rows this far outside the viewport are logged too, so a row entering it has a past.
const MARGIN: f32 = 200.0;

struct Log {
    out: std::io::BufWriter<std::fs::File>,
    key: String,
}

thread_local! {
    static LOG: RefCell<Option<Log>> = RefCell::new(std::env::var_os("RUNE_SCROLL_LOG").and_then(|p| {
        let out = std::io::BufWriter::new(std::fs::File::create(p).ok()?);
        Some(Log { out, key: std::env::var("RUNE_SCROLL_LOG_KEY").unwrap_or_else(|_| "transcript".into()) })
    }));
}

fn unix_ms() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0.0, |d| d.as_secs_f64() * 1000.0)
}

fn on() -> bool { LOG.with(|l| l.borrow().is_some()) }

impl<S: 'static> Runtime<S> {
    /// After a frame's drawn rects are final: where each logged viewport stands.
    pub(super) fn log_scroll_frame(&self) {
        if !on() || self.drawn.len() != self.nodes.len() { return; }
        LOG.with(|l| {
            let mut l = l.borrow_mut();
            let Some(log) = l.as_mut() else { return };
            for s in 0..self.nodes.len() {
                if self.nodes[s].key.as_deref() != Some(log.key.as_str()) || self.nodes[s].scroll.is_none() { continue; }
                let Some(st) = self.scrolls.get(&self.nodes[s].id) else { continue };
                let [_, vy, _, vh] = self.drawn[s];
                let mut rows = Vec::new();
                for c in self.scroll_children(s) {
                    for j in c + 1..self.nodes.len() {
                        if self.nodes[j].parent != Some(c) { continue; }
                        let [_, y, _, h] = self.drawn[j];
                        if h <= 0.0 || y + h < vy - MARGIN || y > vy + vh + MARGIN { continue; }
                        rows.push(format!("[{:?},{:.2},{:.2}]", self.nodes[j].key.as_deref().unwrap_or(""), y - vy, h));
                    }
                }
                let _ = writeln!(log.out, "{{\"e\":\"frame\",\"t\":{:.3},\"off\":{:.2},\"max\":{:.2},\"pinned\":{},\"vh\":{vh:.1},\"rows\":[{}]}}",
                    unix_ms(), st.offset, st.max, st.pinned, rows.join(","));
            }
            let _ = log.out.flush();
        });
    }

    /// A wheel delta `dy` moved viewport `id`.
    pub(super) fn log_scroll_wheel(&self, id: NodeId, dy: f32) {
        if !on() { return; }
        let Some(&i) = self.node_index.get(&id) else { return };
        LOG.with(|l| if let Some(log) = l.borrow_mut().as_mut() {
            let _ = writeln!(log.out, "{{\"e\":\"wheel\",\"t\":{:.3},\"dy\":{dy:.2},\"to\":{:?}}}", unix_ms(), self.nodes[i].path);
        });
    }
}
