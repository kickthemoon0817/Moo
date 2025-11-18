#[derive(Debug, Default)]
pub struct SceneGraph {
    nodes: Vec<SceneNode>,
}

impl SceneGraph {
    pub fn add_node(&mut self, node: SceneNode) {
        tracing::debug!(name = %node.name, "adding scene node");
        self.nodes.push(node);
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

#[derive(Debug, Clone)]
pub struct SceneNode {
    pub name: String,
}

impl SceneNode {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}
