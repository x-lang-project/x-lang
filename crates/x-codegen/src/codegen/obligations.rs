use std::collections::BTreeMap;

use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::values::CallSiteValue;

use super::CodeGeneration;
use x_mir::LoweredMemoryEffects;

impl CodeGeneration<'_> {
    pub(super) fn discharge_lowered_effect_obligations(&self) {
        let mut ledger = self.contract_obligations.borrow_mut();
        let names: BTreeMap<String, String> = ledger
            .functions
            .keys()
            .map(|name| (self.llvm_function_symbol(name).into(), name.clone()))
            .collect();
        let mut calls: BTreeMap<String, Vec<String>> = BTreeMap::new();

        for (name, entry) in &mut ledger.functions {
            let Some(function) = self.module.get_function(self.llvm_function_symbol(name)) else {
                entry.memory.opaque_calls = true;
                continue;
            };

            for block in function.get_basic_blocks() {
                for instruction in block.get_instructions() {
                    let Ok(call) = CallSiteValue::try_from(instruction) else {
                        continue;
                    };

                    let callee = call.get_called_fn_value().and_then(|callee| {
                        callee
                            .get_name()
                            .to_str()
                            .ok()
                            .and_then(|name| names.get(name))
                            .cloned()
                    });

                    if let Some(callee) = callee {
                        calls.entry(name.clone()).or_default().push(callee);
                    } else {
                        entry.memory.opaque_calls = true;
                    }
                }
            }
        }

        loop {
            let previous: BTreeMap<String, LoweredMemoryEffects> = ledger
                .functions
                .iter()
                .map(|(name, entry)| (name.clone(), entry.memory))
                .collect();
            let mut changed = false;

            for (name, entry) in &mut ledger.functions {
                for callee in calls.get(name).into_iter().flatten() {
                    if let Some(effect) = previous.get(callee) {
                        let joined = entry.memory.union(*effect);
                        changed |= joined != entry.memory;
                        entry.memory = joined;
                    }
                }
            }

            if !changed {
                break;
            }
        }

        let memory = Attribute::get_named_enum_kind_id("memory");

        for (name, entry) in &ledger.functions {
            let Some(function) = self.module.get_function(self.llvm_function_symbol(name)) else {
                continue;
            };

            function.remove_enum_attribute(AttributeLoc::Function, memory);

            if entry.source_pure && entry.memory.permits_memory_none() {
                function.add_attribute(
                    AttributeLoc::Function,
                    self.context.create_enum_attribute(memory, 0),
                );
            }
        }
    }
}
