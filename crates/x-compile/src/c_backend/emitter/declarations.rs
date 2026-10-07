use std::collections::BTreeSet;
use std::fmt::Write;

use super::{EmitError, Emitter, Result, ctype, encoded, memoise, string, throws_ctype};
use x_ast::Type;
use x_diagnostics::backend_messages;

#[derive(Default)]
struct TypeDeclarations {
    done: BTreeSet<String>,
    active: BTreeSet<String>,
    source: String,
}

impl Emitter<'_> {
    pub(super) fn declare_functions(&self, declarations: &mut String) -> Result<()> {
        for name in &self.order {
            let text = self.symbols.get(name).ok_or_else(|| {
                EmitError::internal(backend_messages::c_emission_registered_function_is_missing())
            })?;
            let return_value = if *name == "main" {
                "int".to_owned()
            } else if text.body.is_throws {
                throws_ctype(text.return_value)
            } else {
                ctype(text.return_value)?
            };

            writeln!(
                declarations,
                "{return_value} {}({});",
                text.symbol,
                if text.parameters.is_empty() {
                    "void".to_owned()
                } else {
                    text.parameters
                        .iter()
                        .map(ctype)
                        .collect::<Result<Vec<_>>>()?
                        .join(",")
                }
            )?;
        }

        Ok(())
    }

    pub(super) fn string_literals(&self) -> Result<String> {
        let mut literals = String::new();
        let mut strings: Vec<_> = self
            .strings
            .borrow()
            .iter()
            .map(|(text, identifier)| (text.clone(), *identifier))
            .collect();
        strings.sort_by_key(|(_, identifier)| *identifier);
        for (text, identifier) in strings {
            writeln!(
                literals,
                "static struct _x_str _x_literal{identifier}={{(const unsigned char*){},{}ULL}};",
                string(&text),
                text.len()
            )?;
        }

        Ok(literals)
    }
}
