use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
};

mod dump;

pub use dump::AutomationDump;

pub trait Pattern {
    type Char: Eq + Hash;

    fn iter(&self) -> impl Iterator<Item = Self::Char>;
}

impl Pattern for &str {
    type Char = char;

    fn iter(&self) -> impl Iterator<Item = Self::Char> {
        self.chars()
    }
}

impl<P: Pattern> Pattern for &P {
    type Char = P::Char;

    fn iter(&self) -> impl Iterator<Item = Self::Char> {
        (*self).iter()
    }
}

struct AutomationNode<C> {
    goto: HashMap<C, usize>,
    failure: usize,
    outputs: Vec<usize>,
}

impl<C: Eq + Hash> AutomationNode<C> {
    pub fn new() -> Self {
        Self {
            goto: HashMap::new(),
            failure: 0,
            outputs: Vec::new(),
        }
    }

    fn contains(&self, c: &C) -> bool {
        self.goto.contains_key(c)
    }

    fn enter_child(&self, c: &C) -> Option<usize> {
        self.goto.get(c).map(|&x| x)
    }

    fn add_child(&mut self, c: C, node_idx: usize) {
        self.goto.insert(c, node_idx);
    }

    fn add_output(&mut self, output: usize) {
        self.outputs.push(output);
    }
}

pub struct Automation<P: Pattern> {
    nodes: Vec<AutomationNode<P::Char>>,
    output_cnt: usize,
}

impl<P: Pattern> Automation<P> {
    pub fn build(items: impl Iterator<Item = P>) -> Self {
        let mut automation = Automation {
            nodes: Vec::new(),
            output_cnt: 0,
        };

        // Add root node
        automation.nodes.push(AutomationNode::new());

        automation.add_items(items);
        automation.build_failure();

        automation
    }

    fn add_items(&mut self, items: impl Iterator<Item = P>) {
        for item in items {
            self.add_item(item);
        }
    }

    fn add_item(&mut self, item: P) {
        let mut node_idx = 0;

        for c in item.iter() {
            if let Some(n) = self.nodes[node_idx].enter_child(&c) {
                node_idx = n;
            } else {
                let new_node_idx = self.nodes.len();
                self.nodes.push(AutomationNode::new());
                self.nodes[node_idx].add_child(c, new_node_idx);
                node_idx = new_node_idx;
            }
        }

        let output_idx = self.output_cnt;
        self.nodes[node_idx].add_output(output_idx);
        self.output_cnt += 1;
    }

    fn get_node(&self, idx: usize) -> &AutomationNode<P::Char> {
        &self.nodes[idx]
    }

    fn build_failure(&mut self) {
        // Initializes failre function F[i] = lps(i) for each node i that is not root,
        // where lps(i) is the longest proper suffix of node i that is inside the trie.

        // Use BFS to traverse the nodes of the trie in the order of increasing length.
        let mut queue = VecDeque::new();
        queue.push_back(0);

        while let Some(node_index) = queue.pop_front() {
            for (c, &next_node_index) in self.nodes[node_index].goto.iter() {
                let lps = if node_index != 0 {
                    let mut lps = node_index;

                    // Find longest proper suffix for the next node.
                    loop {
                        lps = self.nodes[lps].failure;

                        if lps == 0 || self.nodes[lps].goto.contains_key(c) {
                            break;
                        }
                    }

                    self.nodes[lps].goto.get(c).map(|&x| x).unwrap_or(0)
                } else {
                    // There are no proper suffixes for all nodes
                    // directly accessible from root (the nodes of length 1).
                    // Set failure to the root (0).
                    0
                };

                assert_ne!(next_node_index, lps);
                unsafe {
                    // SAFETY: safe because `next_node_index` != `node_index`.
                    let next_node = &mut *(self.nodes.as_ptr().add(next_node_index) as *const _
                        as *mut AutomationNode<P::Char>);
                    next_node.failure = lps;

                    // Merge outputs with lps
                    let h: HashSet<usize> =
                        HashSet::from_iter(next_node.outputs.iter().map(|&x| x));

                    for &output in &self.nodes[lps].outputs {
                        if !h.contains(&output) {
                            next_node.outputs.push(output);
                        }
                    }
                }

                queue.push_back(next_node_index);
            }
        }
    }

    pub fn dump(&self) -> AutomationDump
    where
        P: ToString,
        P::Char: ToString,
    {
        AutomationDump::create(self)
    }

    pub fn search(&self) -> AutomationSearch<P> {
        AutomationSearch::new(self)
    }
}

pub struct AutomationSearch<'a, P: Pattern> {
    automation: &'a Automation<P>,
    current: usize,
}

impl<'a, P: Pattern> AutomationSearch<'a, P> {
    pub fn new(automation: &'a Automation<P>) -> Self {
        Self {
            automation,
            current: 0,
        }
    }

    pub fn next(&mut self, c: &P::Char) -> &[usize] {
        let mut node = self.automation.get_node(self.current);

        while self.current != 0 && !node.contains(c) {
            self.current = node.failure;
            node = self.automation.get_node(self.current);
        }

        self.current = node.enter_child(c).unwrap_or(0);
        &self.automation.get_node(self.current).outputs
    }
}
