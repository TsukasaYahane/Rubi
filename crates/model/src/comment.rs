use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    pub rpid: u64,
    /// Root comment of the thread; 0 for a root itself.
    pub root: u64,
    /// Direct parent; equals `root` for first-level replies, 0 for a root.
    pub parent: u64,
    pub mid: u64,
    pub name: String,
    pub text: String,
    pub likes: u64,
    pub ctime: i64,
    pub is_up: bool,
    pub reply_count: u32,
}

/// One row of a thread drawn as a fishbone tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreadRow {
    pub rpid: u64,
    /// 0 for the root, 1 for its replies, 2 for replies to those…
    pub depth: u32,
    /// Whether a later sibling (same parent) is visible after this row.
    pub has_later_sibling: bool,
    /// Name of the comment this one replies to, for depth >= 2.
    pub reply_to: Option<String>,
}

/// Rebuilds the reply tree of `root` from the flat reply list and returns its rows in
/// depth-first order, showing at most `limit` replies (the root row is always included).
/// Returns the rows and how many replies were left out.
pub fn flatten_thread(root: &Comment, replies: &[Comment], limit: usize) -> (Vec<ThreadRow>, usize) {
    let mut order: Vec<(&Comment, u32)> = Vec::new();
    walk(root.rpid, 1, replies, &mut order);
    let hidden = order.len().saturating_sub(limit);
    order.truncate(limit);

    let mut rows = vec![ThreadRow { rpid: root.rpid, depth: 0, has_later_sibling: false, reply_to: None }];
    for (i, (c, depth)) in order.iter().enumerate() {
        let has_later_sibling = order[i + 1..].iter().any(|(o, _)| o.parent == c.parent);
        let reply_to =
            if *depth >= 2 { replies.iter().find(|p| p.rpid == c.parent).map(|p| p.name.clone()) } else { None };
        rows.push(ThreadRow { rpid: c.rpid, depth: *depth, has_later_sibling, reply_to });
    }
    (rows, hidden)
}

fn walk<'a>(parent: u64, depth: u32, replies: &'a [Comment], out: &mut Vec<(&'a Comment, u32)>) {
    let mut kids: Vec<&Comment> = replies.iter().filter(|c| c.parent == parent).collect();
    kids.sort_by_key(|c| c.ctime);
    for k in kids {
        out.push((k, depth));
        walk(k.rpid, depth + 1, replies, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(rpid: u64, parent: u64, ctime: i64) -> Comment {
        Comment { rpid, root: 1, parent, ctime, name: format!("u{rpid}"), ..Default::default() }
    }

    #[test]
    fn rebuilds_depth_first_tree() {
        let root = Comment { rpid: 1, ..Default::default() };
        let replies = vec![c(2, 1, 10), c(3, 2, 11), c(4, 1, 12), c(5, 3, 13)];
        let (rows, hidden) = flatten_thread(&root, &replies, 10);
        let got: Vec<(u64, u32, bool)> = rows.iter().map(|r| (r.rpid, r.depth, r.has_later_sibling)).collect();
        assert_eq!(got, vec![(1, 0, false), (2, 1, true), (3, 2, false), (5, 3, false), (4, 1, false)]);
        assert_eq!(rows[2].reply_to.as_deref(), Some("u2"));
        assert_eq!(hidden, 0);
    }

    #[test]
    fn limit_hides_tail_and_recomputes_siblings() {
        let root = Comment { rpid: 1, ..Default::default() };
        let replies = vec![c(2, 1, 10), c(3, 2, 11), c(4, 1, 12)];
        let (rows, hidden) = flatten_thread(&root, &replies, 2);
        assert_eq!(rows.len(), 3);
        assert!(!rows[1].has_later_sibling, "sibling 4 is hidden, so 2 is last visible");
        assert_eq!(hidden, 1);
    }
}
