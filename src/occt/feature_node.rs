use super::FeatureOp;
use std::{
    collections::BTreeSet,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

#[derive(Clone, Debug)]
pub(crate) struct FeatureNode {
    pub(crate) id: u64,
    pub(crate) op: FeatureOp,
    pub(crate) parents: Vec<Arc<FeatureNode>>,
    pub(crate) history_entry: String,
}

static FEATURE_NODE_SEQ: AtomicU64 = AtomicU64::new(1);

impl FeatureNode {
    pub(crate) fn new(
        op: FeatureOp,
        parents: Vec<Arc<FeatureNode>>,
        history_entry: String,
    ) -> Arc<Self> {
        debug_assert!(
            op.parents_are_valid(parents.len()),
            "feature node parent arity mismatch"
        );
        Arc::new(Self {
            id: FEATURE_NODE_SEQ.fetch_add(1, Ordering::Relaxed),
            op,
            parents,
            history_entry,
        })
    }

    fn label(&self) -> String {
        self.op.name()
    }

    pub(crate) fn snapshot_lines(&self, out: &mut Vec<String>, seen: &mut BTreeSet<u64>) {
        if !seen.insert(self.id) {
            return;
        }
        for parent in &self.parents {
            parent.snapshot_lines(out, seen);
        }
        let parent_ids = self
            .parents
            .iter()
            .map(|p| p.id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        out.push(format!(
            "{}\t{}\t{}\t{}",
            self.id,
            parent_ids,
            self.label(),
            self.history_entry.replace('\t', " ")
        ));
    }
}
