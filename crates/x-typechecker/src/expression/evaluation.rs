use super::*;

impl TypeChecker {
    fn register_match_bindings(&mut self, bindings: &[String], elements: &[Type]) {
        for (binding, value_type) in bindings.iter().zip(elements) {
            if binding != "_" {
                self.symbols
                    .add_variable(binding.clone(), value_type.clone());
            }
        }
    }
}
