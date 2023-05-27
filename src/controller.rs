///! The Controller takes the flow definetion as JSON, parses it, and runs the flow
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    thread,
};
use uuid::Uuid;

use crate::{components::TrackList, error::PublicError};

//

#[derive(Clone, PartialEq)]
pub enum Op {
    Gt,
    Lt,
}

#[derive(Clone, PartialEq)]
struct Constraint<T> {
    lhs: T,
    rhs: T,
    op: Op,
}

//

pub type Cache = Arc<RwLock<HashMap<Uuid, TrackList>>>;
pub type Batch = Vec<Uuid>;
pub type Schedule = Vec<Batch>;

//

pub type Edge = (uuid::Uuid, uuid::Uuid);

#[derive(Serialize, Deserialize, Clone)]
pub struct Node {
    pub component: String,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserDefinedFlow {
    pub nodes: HashMap<uuid::Uuid, Node>,
    pub edges: Vec<Edge>,
}

impl UserDefinedFlow {
    fn detect_cycles(&self) -> Result<(), ()> {
        todo!()
    }

    fn build_schedule(&self) -> Result<Schedule, PublicError> {
        let mut constraints = Vec::<Constraint<&Uuid>>::new();
        let mut domains = HashMap::<&Uuid, Vec<usize>>::new();

        for (id, _) in self.nodes.iter() {
            // Build the node domain - This is a simple vector containing
            // every possible index that the node should be run at.
            // E.g. For nodes [A, B, C] the domain for each will be [0, 1, 2].
            domains.insert(id, (0..self.nodes.len()).into_iter().collect());

            // Build the node constraints - If this node dependes on a different node, apply that constraint here.
            // E.g. Edge [A, B] becomes Constraint "A < B" and "B > A"
            for (lhs, rhs) in self.edges.iter() {
                if rhs == id {
                    #[rustfmt::skip]
                    constraints.push(Constraint { lhs, rhs, op: Op::Lt });
                    #[rustfmt::skip]
                    constraints.push(Constraint { lhs: rhs, rhs: lhs, op: Op::Gt });
                }
            }
        }

        // -
        // Reduce the domain using the AC-3 algorithm.
        // Build the inital agenda - We will work through the agenda step-by-step
        let mut agenda: Vec<&Constraint<&Uuid>> = constraints.iter().collect();

        while let Some(constraint) = agenda.pop() {
            let rhs = domains.get(&constraint.rhs).unwrap().clone();
            let lhs = domains.get_mut(&constraint.lhs).unwrap();

            // Keep track of whether the lhs domain has changed -
            // If it does change we need to update the agenda.
            let mut changed = false;

            // Check for each LHS value, there is a RHS value that satifies the constraint.
            // E.g. Given the domains (A = [1,2,3], B = [1,2,3]), verify that each value of A
            //      can satify the constraint "A < B". In this case the the domain of A would be
            //      constrained to [2,3] because "A=1" cannot be less than any value for B.
            lhs.retain(|lhs_value| {
                let mut satified = false;
                for (_, rhs_value) in rhs.iter().enumerate() {
                    satified = match constraint.op {
                        Op::Gt => lhs_value > rhs_value,
                        Op::Lt => lhs_value < rhs_value,
                    };

                    if satified {
                        // Found a rhs_value that satifies the constratint -
                        // Move on to the next lhs_value.
                        break;
                    }
                }
                changed = changed || !satified;
                satified
            });

            if changed {
                // Verify that the domain still has a valid option -
                // If not then this problem is unsolvable.
                if lhs.is_empty() {
                    return Err(format!(
                        "Failed to find a valid constraint for node:{}",
                        constraint.lhs
                    )
                    .into());
                }

                let affected = constraints
                    .iter()
                    .filter(|c| {
                        // Find all constraints with the changed domain that are
                        // not already in the queue.
                        // E.g. With the constraints "A > B" and "B < A", find every constratint with "A" on the right.
                        c.rhs == constraint.lhs && !agenda.contains(c)
                    })
                    .collect::<Vec<_>>();

                // Add the affected constaints to the agenda -
                // These require further processing.
                agenda.extend(affected);
            }
        }

        // --

        let mut schedule: Schedule = Vec::new();

        // Resize the schedule vec with enough space for one node per batch -
        // In many cases this will be overprovisioned, therefore we need to clean up the empty batches at the end.
        schedule.resize_with(self.nodes.len(), || vec![]);

        for (id, domain) in domains {
            // Use the first domain value that applies - TODO: Add backtracking???
            // n.b. Unwrap here is fine because we do `lhs.is_empty()` above.
            let batch_id = *domain.first().unwrap();

            if let Some(batch) = schedule.get_mut(batch_id) {
                batch.push(*id);
            }
        }

        schedule.retain(|b| !b.is_empty());

        Ok(schedule)
    }

    // --

    pub fn execute(&self) -> Result<(), PublicError> {
        let cache = Cache::new(RwLock::new(HashMap::new()));
        for batch in self.build_schedule()?.iter() {
            self.execute_batch(batch, &cache)?;
        }
        Ok(())
    }

    pub fn execute_batch(&self, batch: &Batch, cache: &Cache) -> Result<(), PublicError> {
        thread::scope(|s| {
            let mut handles = Vec::new();

            // Run each node in batch
            for node_id in batch.iter() {
                let node = self.nodes.get(node_id).unwrap();
                let result_cache = Arc::clone(&cache);

                let h = s.spawn(move || {
                    // Do some work 1..2..3..
                    thread::sleep(std::time::Duration::from_millis(500));
                    println!("{}", node.component);

                    // Push results to the cache
                    result_cache.write().unwrap().insert(*node_id, Vec::new());
                });

                handles.push(h);
            }

            // Wait for all nodes in batch to complete
            for h in handles {
                h.join().unwrap();
            }
        });

        Ok(())
    }
}

// --

#[cfg(test)]
mod tests {
    use super::{Schedule, UserDefinedFlow};
    use std::{collections::HashSet, str::FromStr};
    use uuid::Uuid;

    const TEST_YAML: &str = "
---
nodes:
    f0cb5d21-abad-4d11-9dbf-12855a01c463: 
        component: output:overwrite
        parameters:
            by_name: test playlist

    377033c8-c36c-4f04-a716-5e1736f4dfdc: 
        component: combiner:zip

    da0e029b-7a25-424e-b031-fc1271e38069: 
        component: source:user_liked_songs
        parameters:
            id: spotify:user:test

    b38547f9-22cc-47ab-94bb-da695ee3ac4b: 
        component: source:artist_top_tracks
        parameters: 
            id: spotify:artist:6qqNVTkY8uBg9cP3Jd7DAH

    587d87da-0b5b-4b89-a41b-63414b93235c: 
        component: filter:most_recent
        parameters:
            count: 20

    5d83eaac-546e-41f8-b584-9558c037a90c: 
        component: filter:track_deduplication

edges:
    - [da0e029b-7a25-424e-b031-fc1271e38069, 587d87da-0b5b-4b89-a41b-63414b93235c]
    - [587d87da-0b5b-4b89-a41b-63414b93235c, 377033c8-c36c-4f04-a716-5e1736f4dfdc]
    - [b38547f9-22cc-47ab-94bb-da695ee3ac4b, 377033c8-c36c-4f04-a716-5e1736f4dfdc]
    - [377033c8-c36c-4f04-a716-5e1736f4dfdc, 5d83eaac-546e-41f8-b584-9558c037a90c]
    - [5d83eaac-546e-41f8-b584-9558c037a90c, f0cb5d21-abad-4d11-9dbf-12855a01c463]
";

    #[test]
    fn can_parse_user_defined_flow() {
        let _: UserDefinedFlow = serde_yaml::from_str(&TEST_YAML).unwrap();
    }

    #[test]
    fn can_build_valid_schedule() {
        let mut flow: UserDefinedFlow = serde_yaml::from_str(&TEST_YAML).unwrap();
        let mut schedule = flow.build_schedule().unwrap();

        assert_batches(
            schedule,
            &[
                "da0e029b-7a25-424e-b031-fc1271e38069, b38547f9-22cc-47ab-94bb-da695ee3ac4b",
                "587d87da-0b5b-4b89-a41b-63414b93235c",
                "377033c8-c36c-4f04-a716-5e1736f4dfdc",
                "5d83eaac-546e-41f8-b584-9558c037a90c",
                "f0cb5d21-abad-4d11-9dbf-12855a01c463",
            ],
        );
    }

    //

    pub fn assert_batches(schedule: Schedule, expected: &[&str]) {
        for (i, batch) in schedule.iter().enumerate() {
            let expected_nodes: HashSet<Uuid> = expected[i]
                .split(',')
                .map(|id| Uuid::from_str(id.trim()).unwrap())
                .collect();

            let actual_nodes: HashSet<Uuid, _> = HashSet::from_iter(batch.iter().cloned());

            assert_eq!(expected_nodes, actual_nodes);
        }
    }
}
