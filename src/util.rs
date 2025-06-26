use petgraph::{Graph, visit::EdgeRef};

pub fn get_dot_from_petgraph<T>(graph: &Graph<T, ()>) -> String
where
    T: std::fmt::Display,
{
    let mut rv = String::new();
    rv.push_str("digraph G {\n\n");

    for node_idx in graph.node_indices() {
        let node_data = graph.node_weight(node_idx).unwrap();
        rv.push_str(&format!("{}[label=\"{}\"];\n", node_idx.index(), node_data));
    }
    for edge in graph.edge_references() {
        let source = edge.source().index();
        let target = edge.target().index();
        rv.push_str(&format!("{} -> {};\n", source, target));
    }
    rv.push_str("\n}");
    rv
}
