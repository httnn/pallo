use std::collections::VecDeque;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct NodeId {
    pub id: usize,
}

pub struct Node<T> {
    data: T,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
}

pub struct Tree<T> {
    nodes: Vec<Node<T>>,
    empty_cells: Vec<usize>,
    root_id: NodeId,
}

impl<T: Default> Default for Tree<T> {
    fn default() -> Self {
        let root = Node { data: T::default(), parent: None, children: vec![] };
        Self { nodes: vec![root], root_id: NodeId { id: 0 }, empty_cells: vec![] }
    }
}

impl<T: Default> Tree<T> {
    pub fn add(&mut self, parent_id: NodeId) -> NodeId {
        let node = Node { data: T::default(), parent: Some(parent_id), children: vec![] };
        let id = if let Some(idx) = self.empty_cells.pop() {
            self.nodes[idx] = node;
            NodeId { id: idx }
        } else {
            self.nodes.push(node);
            NodeId { id: self.nodes.len() - 1 }
        };
        self.nodes.get_mut(parent_id.id).unwrap().children.push(id);
        id
    }

    pub fn traverse_depth(&self, root: NodeId, mut callback: impl FnMut(NodeId, &T) -> bool) {
        let mut stack = Vec::new();
        stack.push(root);

        while let Some(current) = stack.pop() {
            debug_assert!(current.id < self.nodes.len());
            let node = unsafe { self.nodes.get_unchecked(current.id) };
            if callback(current, &node.data) || current.id == root.id {
                for child in node.children.iter().rev() {
                    stack.push(*child);
                }
            }
        }
    }

    pub fn traverse_depth_mut(&mut self, root: NodeId, mut callback: impl FnMut(NodeId, &mut T) -> bool) {
        let mut stack = Vec::new();
        stack.push(root);

        while let Some(current) = stack.pop() {
            debug_assert!(current.id < self.nodes.len());
            let node = unsafe { self.nodes.get_unchecked_mut(current.id) };
            if callback(current, &mut node.data) || current.id == root.id {
                for child in node.children.iter().rev() {
                    stack.push(*child);
                }
            }
        }
    }

    #[allow(unused)]
    pub fn traverse_breadth<F: FnMut(NodeId, &T) -> bool>(&self, root: NodeId, mut callback: F) {
        let mut queue = VecDeque::new();
        queue.push_back(root);

        while let Some(current) = queue.pop_front() {
            if (callback(current, self.get(current)))
                && let Some(node) = self.nodes.get(current.id)
            {
                for child in &node.children {
                    queue.push_back(*child);
                }
            }
        }
    }

    // should remove the node specified by the id along with its entire subtree
    pub fn remove(&mut self, id: NodeId) {
        if let Some(parent_id) = self.nodes[id.id].parent {
            let children = &mut self.nodes.get_mut(parent_id.id).unwrap().children;
            let index = children.iter().position(|x| *x == id).unwrap();
            children.remove(index);
        }
        self.empty_cells.push(id.id);
    }

    pub fn get_children_mut(&mut self, id: NodeId) -> &mut Vec<NodeId> {
        &mut self.nodes[id.id].children
    }

    pub fn get_root_id(&self) -> NodeId {
        self.root_id
    }

    pub fn get_parent(&self, id: NodeId) -> Option<NodeId> {
        self.nodes[id.id].parent
    }

    pub fn get_mut(&mut self, id: NodeId) -> &mut T {
        debug_assert!(id.id < self.nodes.len());
        unsafe { &mut self.nodes.get_unchecked_mut(id.id).data }
    }

    pub fn get(&self, id: NodeId) -> &T {
        debug_assert!(id.id < self.nodes.len());
        unsafe { &self.nodes.get_unchecked(id.id).data }
    }
}
