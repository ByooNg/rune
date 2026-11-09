//! A press on the bare space of a selection group (a margin, the gap between paragraphs) starts
//! the selection at the group's nearest text, as a web page does, so a drag begun beside the
//! text still selects it.
use super::selectable::dist_x;
use super::*;

impl<S: 'static> Runtime<S> {
    /// For a press at (x, y) on nothing that takes it, inside a selection group: the group's text
    /// nearest the point and the byte there. The innermost group around the point answers; its
    /// texts on the press's line take the one nearest across, otherwise the line nearest above
    /// or below, so a press in the margin beside a paragraph starts at that paragraph's edge.
    pub(super) fn group_text_near(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        if self.top_target(x, y).is_some() {
            return None;
        }
        let g = (0..self.nodes.len())
            .filter(|&g| self.nodes[g].selection_group && !self.subtree_exiting(g) && self.allowed(g))
            .filter(|&g| {
                let r = self.drawn_box(g);
                x >= r[0] && x < r[2] && y >= r[1] && y < r[3]
            })
            .min_by(|&a, &b| {
                let area = |r: [f32; 4]| (r[2] - r[0]) * (r[3] - r[1]);
                area(self.drawn_box(a)).total_cmp(&area(self.drawn_box(b)))
            })?;
        let dist_y = |r: [f32; 4]| (r[1] - y).max(y - r[3]).max(0.0);
        let (j, r) = self.group_texts(g).into_iter().filter_map(|j| Some((j, self.shown_box(j)?)))
            .min_by(|p, q| (dist_y(p.1), dist_x(p.1, x)).partial_cmp(&(dist_y(q.1), dist_x(q.1, x))).unwrap_or(std::cmp::Ordering::Equal))?;
        // Inside the text's box, so the press lands on its nearest line and character.
        let (cx, cy) = (x.clamp(r[0], (r[2] - 0.01).max(r[0])), y.clamp(r[1], (r[3] - 0.01).max(r[1])));
        Some((j, self.text_index_at(j, cx, cy)?))
    }
}
