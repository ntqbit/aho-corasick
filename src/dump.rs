use crate::{Automation, Pattern};

#[derive(Clone)]
struct AutomationDumpNode {
    node: String,
    goto: Vec<usize>,
    failure: usize,
    outputs: Vec<usize>,
}

#[derive(Clone)]
enum EdTarget {
    Goto(usize),
    Failure(usize),
}

type EdgeDesc = (usize, EdTarget);

pub struct AutomationDump {
    nodes: Vec<AutomationDumpNode>,
    edges: Vec<EdgeDesc>,
    outputs: Vec<String>,
}

impl AutomationDump {
    pub fn create<P>(automation: &Automation<P>) -> Self
    where
        P: Pattern + ToString,
        P::Char: ToString,
    {
        let outputs = automation.outputs.iter().map(|x| x.to_string()).collect();
        let mut nodes: Vec<AutomationDumpNode> = automation
            .nodes
            .iter()
            .map(|x| AutomationDumpNode {
                node: String::new(),
                goto: x.goto.values().map(|&x| x).collect(),
                failure: x.failure,
                outputs: x.outputs.clone(),
            })
            .collect();
        let mut edges = Vec::new();

        for (idx, node) in automation.nodes.iter().enumerate() {
            for (c, &next_node) in node.goto.iter() {
                nodes[next_node].node = c.to_string();
                edges.push((idx, EdTarget::Goto(next_node)));
            }
            edges.push((idx, EdTarget::Failure(node.failure)));
        }

        Self {
            nodes,
            outputs,
            edges,
        }
    }
}

#[cfg(feature = "dot")]
mod dotdump {
    use std::io;

    use super::{AutomationDump, EdTarget};

    type Nd = usize;
    type Ed = super::EdgeDesc;

    impl AutomationDump {
        pub fn to_dot(&self) -> io::Result<String> {
            let mut out = Vec::new();
            dot::render(self, &mut out)?;
            Ok(String::from_utf8(out).unwrap())
        }
    }

    impl<'a> dot::Labeller<'a, Nd, Ed> for AutomationDump {
        fn graph_id(&'a self) -> dot::Id<'a> {
            dot::Id::new("automation").unwrap()
        }

        fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
            dot::Id::new(format!("N{}", n)).unwrap()
        }

        fn node_label(&'a self, n: &Nd) -> dot::LabelText<'a> {
            let node = &self.nodes[*n];
            let mut s = node.node.clone();

            if !node.outputs.is_empty() {
                s.push_str(r#"<font point-size="10">"#);

                for &output in &node.outputs {
                    s.push_str("<br/>");
                    s.push_str(&self.outputs[output]);
                }

                s.push_str("</font>");
            }

            if s.is_empty() {
                dot::LabelText::label("")
            } else {
                dot::LabelText::html(s)
            }
        }

        fn edge_style(&'a self, e: &Ed) -> dot::Style {
            match &e.1 {
                EdTarget::Goto(_) => dot::Style::Solid,
                EdTarget::Failure(_) => dot::Style::Dashed,
            }
        }
    }

    impl<'a> dot::GraphWalk<'a, Nd, Ed> for AutomationDump {
        fn nodes(&'a self) -> dot::Nodes<'a, Nd> {
            dot::Nodes::Owned((0..self.nodes.len()).collect())
        }

        fn edges(&'a self) -> dot::Edges<'a, Ed> {
            dot::Nodes::Borrowed(&self.edges)
        }

        fn source(&'a self, edge: &Ed) -> Nd {
            edge.0
        }

        fn target(&'a self, edge: &Ed) -> Nd {
            match &edge.1 {
                EdTarget::Goto(idx) => *idx,
                EdTarget::Failure(idx) => *idx,
            }
        }
    }
}
