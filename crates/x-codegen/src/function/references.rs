use inkwell::values::BasicValueEnum;
use inkwell::{AddressSpace, targets, values};

use super::FunctionCompiler;
use crate::{codegen, errors::CodeGenerationError};
use x_ast::{ReferenceQualifiers, Type};
use x_diagnostics::backend_messages;
use x_mir::{Operand, Place};

impl<'ctx> FunctionCompiler<'_, 'ctx> {
    fn cheri_projection(
        &self,
        place: &Place,
        target_data: &targets::TargetData,
    ) -> Result<(String, Type, Option<u64>), CodeGenerationError> {
        match place {
            Place::Local(identifier) => {
                let name = self
                    .body
                    .locals
                    .get(identifier)
                    .and_then(|declaration| declaration.name.clone())
                    .unwrap_or_else(|| format!("_{}", identifier.0));

                Ok((name, self.get_place_type(place), Some(0)))
            }
            Place::Global { name, value_type } => {
                Ok((format!("global::{name}"), value_type.clone(), Some(0)))
            }
            Place::Dereference { base, .. } => {
                let (path, _, _) = self.cheri_projection(base, target_data)?;

                Ok((format!("*{path}"), self.get_place_type(place), Some(0)))
            }
            Place::Index { base, index, .. } => {
                let (base_path, root_type, base_offset) =
                    self.cheri_projection(base, target_data)?;
                let index_value = match index {
                    Operand::ConstantInteger(value) if *value >= 0 => Some(*value as u64),
                    _ => None,
                };

                let element_type = self.get_place_type(place);
                let element_llvm_type = self.codegen.resolve_llvm_type(&element_type)?;
                let element_size = target_data.get_store_size(&element_llvm_type);
                let offset = base_offset
                    .zip(index_value)
                    .and_then(|(base, index)| index.checked_mul(element_size)?.checked_add(base));
                let index_text = index_value
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "?".to_string());

                Ok((format!("{base_path}[{index_text}]"), root_type, offset))
            }
            Place::Field {
                base, field_name, ..
            } => {
                let (base_path, root_type, base_offset) =
                    self.cheri_projection(base, target_data)?;
                let mut base_type = self.get_place_type(base);

                while let Type::Reference { inner, .. } | Type::RawPointer { inner, .. } = base_type
                {
                    base_type = *inner;
                }

                let field_index = match &base_type {
                    Type::Tuple(_) => field_name.parse::<u32>().ok(),
                    _ => None,
                };

                let field_offset = field_index.and_then(|index| {
                    let llvm_type = self.codegen.resolve_llvm_type(&base_type).ok()?;

                    if !llvm_type.is_struct_type() {
                        return None;
                    }

                    target_data.offset_of_element(&llvm_type.into_struct_type(), index)
                });
                let offset = base_offset
                    .zip(field_offset)
                    .and_then(|(base, field)| base.checked_add(field));

                Ok((format!("{base_path}.{field_name}"), root_type, offset))
            }
        }
    }
}

fn sanitize_trace_field(value: &str) -> String {
    value.replace(['\t', '\r', '\n'], "_")
}
