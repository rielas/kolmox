//! Hand-rolled vantage-point tree with **early-stop** existence search.
//!
//! The external `vp_tree` crate always traverses the whole candidate region to
//! return the complete neighbour set (or the proven nearest), with no way to
//! halt at the first in-range hit. For this benchmark we only care whether
//! *any* page lies within the tolerance, so we roll our own tree whose query
//! returns the moment it finds one. Distances are scaled integers produced by
//! the same closure that drives the BK tree, so the two are directly
//! comparable.

struct Node {
    item: usize,
    /// Median distance from `item` to the points in this subtree.
    threshold: isize,
    /// Points within `threshold` of `item`.
    inside: Option<Box<Node>>,
    /// Points at or beyond `threshold` of `item`.
    outside: Option<Box<Node>>,
}

pub struct VpTree<F> {
    root: Option<Box<Node>>,
    dist: F,
}

impl<F: Fn(&usize, &usize) -> isize> VpTree<F> {
    /// Build a VP tree over `items` using `dist` as the metric.
    pub fn new(items: impl IntoIterator<Item = usize>, dist: F) -> Self {
        let items: Vec<usize> = items.into_iter().collect();
        let mut tree = VpTree { root: None, dist };
        tree.root = tree.build(&items);
        tree
    }

    fn build(&self, items: &[usize]) -> Option<Box<Node>> {
        let (&vp, rest) = items.split_first()?;
        if rest.is_empty() {
            return Some(Box::new(Node {
                item: vp,
                threshold: 0,
                inside: None,
                outside: None,
            }));
        }

        // Distance from the vantage point to every other point, partitioned
        // around the median: closer points go inside, the rest go outside.
        let mut keyed: Vec<(isize, usize)> =
            rest.iter().map(|&it| ((self.dist)(&vp, &it), it)).collect();
        let mid = keyed.len() / 2;
        keyed.select_nth_unstable_by_key(mid, |&(d, _)| d);
        let threshold = keyed[mid].0;

        let inside: Vec<usize> = keyed[..mid].iter().map(|&(_, it)| it).collect();
        let outside: Vec<usize> = keyed[mid..].iter().map(|&(_, it)| it).collect();

        Some(Box::new(Node {
            item: vp,
            threshold,
            inside: self.build(&inside),
            outside: self.build(&outside),
        }))
    }

    /// Return the first item found within `radius` of `query`, stopping as soon
    /// as one is located. Returns `None` if nothing is within `radius`.
    pub fn find_first_within(&self, query: usize, radius: isize) -> Option<(usize, isize)> {
        self.root
            .as_deref()
            .and_then(|n| self.search(n, query, radius))
    }

    fn search(&self, node: &Node, query: usize, radius: isize) -> Option<(usize, isize)> {
        let d = (self.dist)(&query, &node.item);
        if d <= radius {
            return Some((node.item, d)); // early stop
        }

        // Triangle-inequality pruning. Visit the branch the query falls in first
        // so a hit, if any, surfaces sooner.
        let inside_ok = d - radius <= node.threshold;
        let outside_ok = d + radius >= node.threshold;
        let (first, first_ok, second, second_ok) = if d <= node.threshold {
            (&node.inside, inside_ok, &node.outside, outside_ok)
        } else {
            (&node.outside, outside_ok, &node.inside, inside_ok)
        };

        if first_ok {
            if let Some(child) = first {
                if let Some(hit) = self.search(child, query, radius) {
                    return Some(hit);
                }
            }
        }
        if second_ok {
            if let Some(child) = second {
                if let Some(hit) = self.search(child, query, radius) {
                    return Some(hit);
                }
            }
        }
        None
    }
}
