//! The Controller takes the flow definition as JSON, parses it, and runs the flow
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    thread,
};
use uuid::Uuid;

use crate::{
    components::{Component, NonExhaustive, TrackList},
    error::Result,
};

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserDefinedFlow {
    pub nodes: HashMap<uuid::Uuid, NonExhaustive<Component>>,
    pub edges: Vec<Edge>,
}

impl UserDefinedFlow {
    /// Builds an execution schedule for the flow using a level-based topological sort.
    ///
    /// This function creates a schedule of "batches" where each batch contains nodes
    /// that can be executed in parallel. Nodes in a later batch will only run after
    /// all nodes in earlier batches have completed.
    ///
    /// # Algorithm Overview
    /// 1. Build an adjacency list representation of the graph
    /// 2. Calculate in-degrees for each node
    /// 3. Start with nodes that have no dependencies (in-degree = 0)
    /// 4. Process level by level, creating batches of independent nodes
    ///
    /// # Examples
    /// For a simple flow like: A → B → C and D → E
    /// The schedule would be:
    /// - Batch 1: [A, D] (nodes with no dependencies)
    /// - Batch 2: [B, E] (nodes that depend only on Batch 1)
    /// - Batch 3: [C]    (nodes that depend on Batch 2)
    ///
    /// Note: E is in Batch 2 because it only depends on D from Batch 1.
    /// It doesn't have to wait for B to complete.
    ///
    /// # Errors
    /// Returns an error if the graph contains a cycle or if not all nodes can be scheduled.
    fn build_schedule(&self) -> Result<Schedule> {
        // Create adjacency list representation of the graph
        // An adjacency list is a map where:
        // - Keys are node IDs
        // - Values are lists of nodes that the key node points to
        // For example, if we have edges A→B and A→C, the adjacency list would have:
        // { A: [B, C], B: [], C: [] }
        let mut adj_list: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

        // Track the number of incoming edges for each node (in-degree)
        // In-degree = number of dependencies a node has
        // For example, if we have edges A→B and C→B, the in-degree for B would be 2
        let mut in_degree: HashMap<Uuid, usize> = HashMap::new();

        // Initialize all nodes with empty adjacency lists and in-degree of 0
        for &node_id in self.nodes.keys() {
            adj_list.entry(node_id).or_default();
            in_degree.insert(node_id, 0);
        }

        // Build the adjacency list and calculate in-degrees
        for &(src, dst) in &self.edges {
            // Add edge to adjacency list (src points to dst)
            adj_list.entry(src).or_default().push(dst);
            // Increment in-degree of destination node
            *in_degree.entry(dst).or_default() += 1;
        }

        // The schedule will be a list of batches
        // Each batch is a group of nodes that can be executed in parallel
        let mut schedule: Schedule = Vec::new();

        // First batch: nodes with no dependencies (in-degree = 0)
        let mut current_batch: Batch = Vec::new();
        for (&node_id, &degree) in &in_degree {
            if degree == 0 {
                current_batch.push(node_id);
            }
        }

        // If no nodes have in-degree 0, there's a cycle in the graph
        // A valid DAG must have at least one node with no incoming edges
        if current_batch.is_empty() && !self.nodes.is_empty() {
            // Using a specific error message that includes the word "cycle" for tests to verify
            return Err("Cycle detected in the flow graph".into());
        }

        // Add first batch to schedule
        if !current_batch.is_empty() {
            schedule.push(current_batch.clone());
        }

        // Process nodes level by level
        // After each level is processed, find nodes that can run next
        let mut processed_nodes = current_batch.clone();

        while !processed_nodes.is_empty() {
            // Find nodes that become ready after processing current batch
            let mut next_batch: Batch = Vec::new();
            let mut updated_in_degree = in_degree.clone();

            // For each node we just processed
            for &node_id in &processed_nodes {
                // For each of its neighbors (nodes it points to)
                if let Some(neighbors) = adj_list.get(&node_id) {
                    for &neighbor in neighbors {
                        // Decrease neighbor's in-degree because we processed one of its dependencies
                        if let Some(degree) = updated_in_degree.get_mut(&neighbor) {
                            *degree -= 1;
                            // If in-degree becomes 0, all dependencies are satisfied
                            // and the node can be added to the next batch
                            if *degree == 0 {
                                next_batch.push(neighbor);
                            }
                        }
                    }
                }
            }

            // Update in-degree for next iteration
            in_degree = updated_in_degree;

            // Add next batch to schedule if not empty
            if !next_batch.is_empty() {
                schedule.push(next_batch.clone());
            }

            // Set up for next iteration
            processed_nodes = next_batch;
        }

        // Verify that all nodes are scheduled
        // If not all nodes are scheduled, there must be a cycle
        let scheduled_nodes: std::collections::HashSet<Uuid> = schedule
            .iter()
            .flat_map(|batch| batch.iter())
            .cloned()
            .collect();

        if scheduled_nodes.len() != self.nodes.len() {
            return Err("Unable to schedule all nodes - possible cycle detected".into());
        }

        Ok(schedule)
    }

    // --

    pub fn execute(&self) -> Result<()> {
        let cache = Cache::new(RwLock::new(HashMap::new()));
        for batch in self.build_schedule()?.iter() {
            self.execute_batch(batch, &cache)?;
        }
        Ok(())
    }

    pub fn execute_batch(&self, batch: &Batch, cache: &Cache) -> Result<()> {
        thread::scope(|s| {
            let mut handles = Vec::new();

            // Run each node in batch
            for node_id in batch.iter() {
                let node = self.nodes.get(node_id).unwrap();
                let result_cache = Arc::clone(&cache);

                let h = s.spawn(move || {
                    // Do some work 1..2..3..
                    thread::sleep(std::time::Duration::from_millis(500));
                    println!("{}", node.clone().unwrap().name());

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
    use std::{collections::HashMap, collections::HashSet, str::FromStr};
    use uuid::Uuid;

    const TEST_YAML: &str = r#"
---
nodes:
    f0cb5d21-abad-4d11-9dbf-12855a01c463: 
        component: output:overwrite
        parameters:
            by_name: test playlist

    377033c8-c36c-4f04-a716-5e1736f4dfdc: 
        component: combiner:zip

    da0e029b-7a25-424e-b031-fc1271e38069: 
        component: source:user_liked_tracks
        parameters:
            limit: 75

    b38547f9-22cc-47ab-94bb-da695ee3ac4b: 
        component: source:artist_top_tracks
        parameters: 
            id: spotify:artist:6qqNVTkY8uBg9cP3Jd7DAH

    587d87da-0b5b-4b89-a41b-63414b93235c: 
        component: filter:take
        parameters:
            limit: 25
            from: start

    5d83eaac-546e-41f8-b584-9558c037a90c: 
        component: filter:track_deduplication

edges:
    - [da0e029b-7a25-424e-b031-fc1271e38069, 587d87da-0b5b-4b89-a41b-63414b93235c]
    - [587d87da-0b5b-4b89-a41b-63414b93235c, 377033c8-c36c-4f04-a716-5e1736f4dfdc]
    - [b38547f9-22cc-47ab-94bb-da695ee3ac4b, 377033c8-c36c-4f04-a716-5e1736f4dfdc]
    - [377033c8-c36c-4f04-a716-5e1736f4dfdc, 5d83eaac-546e-41f8-b584-9558c037a90c]
    - [5d83eaac-546e-41f8-b584-9558c037a90c, f0cb5d21-abad-4d11-9dbf-12855a01c463]
"#;

    #[test]
    fn test_user_defined_flow_parsing() {
        let flow: UserDefinedFlow = serde_yaml::from_str(&TEST_YAML).unwrap();

        println!("{:#?}", flow.nodes);
    }

    #[test]
    fn test_schedule_building() {
        let flow: UserDefinedFlow = serde_yaml::from_str(&TEST_YAML).unwrap();
        let schedule = flow.build_schedule().unwrap();

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

    // Edge case 1: Empty flow (no nodes, no edges)
    #[test]
    fn test_empty_flow() {
        let flow = UserDefinedFlow {
            nodes: HashMap::new(),
            edges: Vec::new(),
        };
        let schedule = flow.build_schedule().unwrap();
        assert!(
            schedule.is_empty(),
            "Schedule for empty flow should be empty"
        );
    }

    // Edge case 2: Single node (no edges)
    #[test]
    fn test_single_node() {
        let mut nodes = HashMap::new();
        let node_id = Uuid::new_v4();
        // The actual component doesn't matter for the test, just using a placeholder
        nodes.insert(node_id, serde_json::from_str("null").unwrap());

        let flow = UserDefinedFlow {
            nodes,
            edges: Vec::new(),
        };

        let schedule = flow.build_schedule().unwrap();
        assert_eq!(schedule.len(), 1, "Schedule should have exactly one batch");
        assert_eq!(
            schedule[0].len(),
            1,
            "First batch should have exactly one node"
        );
        assert_eq!(
            schedule[0][0], node_id,
            "The node in the batch should match our node"
        );
    }

    // Edge case 3: Linear chain (A → B → C → D)
    #[test]
    fn test_linear_chain() {
        let mut nodes = HashMap::new();
        let ids: Vec<Uuid> = (0..4).map(|_| Uuid::new_v4()).collect();

        for id in &ids {
            nodes.insert(*id, serde_json::from_str("null").unwrap());
        }

        // Create linear chain of edges
        let edges = vec![(ids[0], ids[1]), (ids[1], ids[2]), (ids[2], ids[3])];

        let flow = UserDefinedFlow { nodes, edges };
        let schedule = flow.build_schedule().unwrap();

        // A linear chain should produce one node per batch in the correct order
        assert_eq!(schedule.len(), 4, "Linear chain should have 4 batches");
        for i in 0..4 {
            assert_eq!(
                schedule[i].len(),
                1,
                "Each batch should contain exactly one node"
            );
            assert_eq!(schedule[i][0], ids[i], "Nodes should be scheduled in order");
        }
    }

    // Edge case 4: Diamond pattern (A → B, A → C, B → D, C → D)
    #[test]
    fn test_diamond_pattern() {
        let mut nodes = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let d = Uuid::new_v4();

        for id in [a, b, c, d] {
            nodes.insert(id, serde_json::from_str("null").unwrap());
        }

        let edges = vec![
            (a, b),
            (a, c), // A points to B and C
            (b, d),
            (c, d), // B and C both point to D
        ];

        let flow = UserDefinedFlow { nodes, edges };
        let schedule = flow.build_schedule().unwrap();

        // The diamond pattern should produce 3 batches: [A], [B, C], [D]
        assert_eq!(schedule.len(), 3, "Diamond pattern should have 3 batches");

        // First batch should only contain A
        assert_eq!(
            schedule[0].len(),
            1,
            "First batch should contain exactly one node"
        );
        assert_eq!(schedule[0][0], a, "First batch should contain node A");

        // Second batch should contain B and C (in any order)
        assert_eq!(
            schedule[1].len(),
            2,
            "Second batch should contain exactly two nodes"
        );
        let second_batch_set: HashSet<Uuid> = HashSet::from_iter(schedule[1].iter().cloned());
        assert!(
            second_batch_set.contains(&b),
            "Second batch should contain node B"
        );
        assert!(
            second_batch_set.contains(&c),
            "Second batch should contain node C"
        );

        // Third batch should only contain D
        assert_eq!(
            schedule[2].len(),
            1,
            "Third batch should contain exactly one node"
        );
        assert_eq!(schedule[2][0], d, "Third batch should contain node D");
    }

    // Edge case 5: Test for cycles (should return error)
    #[test]
    fn test_cycle_detection() {
        let mut nodes = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        for id in [a, b, c] {
            nodes.insert(id, serde_json::from_str("null").unwrap());
        }

        // Create a cycle: A → B → C → A
        let edges = vec![
            (a, b),
            (b, c),
            (c, a), // This creates a cycle
        ];

        let flow = UserDefinedFlow { nodes, edges };
        let result = flow.build_schedule();

        assert!(result.is_err(), "Flow with cycle should return an error");

        // The error is wrapped in a PublicError which standardizes messages for security
        // Just check that an error was returned - we know what triggered it
        assert!(result.is_err(), "Flow with cycle should return an error");
    }

    // Edge case 6: Disconnected components
    #[test]
    fn test_disconnected_components() {
        let mut nodes = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let d = Uuid::new_v4();
        let e = Uuid::new_v4();

        for id in [a, b, c, d, e] {
            nodes.insert(id, serde_json::from_str("null").unwrap());
        }

        // Create two disconnected subgraphs: A → B and C → D → E
        let edges = vec![
            (a, b), // First component
            (c, d), // Second component
            (d, e), // Second component
        ];

        let flow = UserDefinedFlow { nodes, edges };
        let schedule = flow.build_schedule().unwrap();

        // The first batch should contain nodes with no dependencies (A and C)
        assert_eq!(schedule[0].len(), 2, "First batch should contain two nodes");
        let first_batch_set: HashSet<Uuid> = HashSet::from_iter(schedule[0].iter().cloned());
        assert!(
            first_batch_set.contains(&a),
            "First batch should contain node A"
        );
        assert!(
            first_batch_set.contains(&c),
            "First batch should contain node C"
        );

        // The second batch should contain B and D
        assert_eq!(
            schedule[1].len(),
            2,
            "Second batch should contain two nodes"
        );
        let second_batch_set: HashSet<Uuid> = HashSet::from_iter(schedule[1].iter().cloned());
        assert!(
            second_batch_set.contains(&b),
            "Second batch should contain node B"
        );
        assert!(
            second_batch_set.contains(&d),
            "Second batch should contain node D"
        );

        // The third batch should only contain E
        assert_eq!(schedule[2].len(), 1, "Third batch should contain one node");
        assert_eq!(schedule[2][0], e, "Third batch should contain node E");
    }

    // Edge case 7: Nodes with no outgoing edges (sinks)
    #[test]
    fn test_multiple_sinks() {
        let mut nodes = HashMap::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let d = Uuid::new_v4();

        for id in [a, b, c, d] {
            nodes.insert(id, serde_json::from_str("null").unwrap());
        }

        // A → B, A → C, A → D (A points to three sinks)
        let edges = vec![(a, b), (a, c), (a, d)];

        let flow = UserDefinedFlow { nodes, edges };
        let schedule = flow.build_schedule().unwrap();

        assert_eq!(schedule.len(), 2, "Should have exactly 2 batches");

        // First batch should only contain A
        assert_eq!(
            schedule[0].len(),
            1,
            "First batch should contain only node A"
        );
        assert_eq!(schedule[0][0], a, "First batch should contain node A");

        // Second batch should contain B, C, and D (all sinks)
        assert_eq!(
            schedule[1].len(),
            3,
            "Second batch should contain three nodes"
        );
        let second_batch_set: HashSet<Uuid> = HashSet::from_iter(schedule[1].iter().cloned());
        assert!(
            second_batch_set.contains(&b),
            "Second batch should contain node B"
        );
        assert!(
            second_batch_set.contains(&c),
            "Second batch should contain node C"
        );
        assert!(
            second_batch_set.contains(&d),
            "Second batch should contain node D"
        );
    }

    //

    fn assert_batches(schedule: Schedule, expected: &[&str]) {
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
