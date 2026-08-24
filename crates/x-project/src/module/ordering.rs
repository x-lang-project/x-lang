use super::*;

impl ModuleResolver {
    pub fn compute_sort_order(&mut self) -> Result<(), CompilationFailure> {
        let mut traversal = SortTraversal::default();

        let mut keys: Vec<String> = self.modules.keys().cloned().collect();
        keys.sort();
        for key in keys {
            self.topological_sort(&key, &mut traversal)?;
        }

        self.order = traversal.order;

        Ok(())
    }

    pub(super) fn topological_sort(
        &self,
        node: &str,
        traversal: &mut SortTraversal,
    ) -> Result<(), CompilationFailure> {
        if traversal.active.contains(node) {
            return Err(ProjectError::module_cycle(node));
        }

        if !traversal.visited.contains(node) {
            traversal.active.insert(node.to_string());

            if let Some(mod_node) = self.modules.get(node) {
                for sub in &mod_node.submodules {
                    let child_node = format!("{node}::{sub}");
                    self.topological_sort(&child_node, traversal)?;
                }
            }

            traversal.active.remove(node);
            traversal.visited.insert(node.to_string());
            traversal.order.push(node.to_string());
        }

        Ok(())
    }
}
