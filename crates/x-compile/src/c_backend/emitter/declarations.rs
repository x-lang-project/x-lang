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
    fn declare_type(&self, value_type: &Type, declarations: &mut TypeDeclarations) -> Result<()> {
        let key = ctype(value_type)?;

        if declarations.done.contains(&key) {
            return Ok(());
        }

        if let Type::RawPointer { inner, .. } | Type::Reference { inner, .. } = value_type {
            if matches!(inner.as_ref(), Type::Tuple(_) | Type::FixedSizedArray(_, _)) {
                self.declare_type(inner, declarations)?;
            }

            return Ok(());
        }

        if matches!(value_type, Type::String) {
            return Ok(());
        }

        if !matches!(
            value_type,
            Type::Custom(_) | Type::Tuple(_) | Type::FixedSizedArray(_, _)
        ) {
            return Ok(());
        }

        if !declarations.active.insert(key.clone()) {
            return Err(backend_messages::c_emission_recursive_value_aggregate_layout().into());
        }

        let fields: Vec<(String, Type)> = match value_type {
            Type::Custom(count) => self
                .structs
                .get(count.as_str())
                .ok_or_else(backend_messages::c_emission_missing_record_definition)?
                .fields
                .iter()
                .map(|(count, type_value)| (format!("f{}", encoded(count)), type_value.clone()))
                .collect(),
            Type::Tuple(items) => items
                .iter()
                .enumerate()
                .map(|(index, type_value)| (format!("f{index}"), type_value.clone()))
                .collect(),
            Type::FixedSizedArray(inner, count) => {
                self.declare_type(inner, declarations)?;
                writeln!(
                    declarations.source,
                    "{key}{{{} data[{count}];}};",
                    ctype(inner)?
                )?;
                declarations.active.remove(&key);
                declarations.done.insert(key);

                return Ok(());
            }
            _ => {
                return Err(EmitError::internal(
                    backend_messages::c_emission_aggregate_type_classification_changed(),
                ));
            }
        };

        for (_, type_value) in &fields {
            self.declare_type(type_value, declarations)?;
        }

        write!(declarations.source, "{key}{{")?;
        for (count, type_value) in &fields {
            write!(declarations.source, "{} {count};", ctype(type_value)?)?;
        }

        declarations.source.push_str("};\n");
        declarations.active.remove(&key);
        declarations.done.insert(key);

        Ok(())
    }

    pub(super) fn type_declarations(&self) -> Result<String> {
        let mut declarations = TypeDeclarations {
            source: String::from(
                "struct _x_str {const unsigned char *data; unsigned long long length;};\n",
            ),
            ..Default::default()
        };
        let mut records: Vec<_> = self.structs.values().copied().collect();
        records.sort_by_key(|definition| &definition.name);
        for current_definition in &records {
            writeln!(
                declarations.source,
                "struct _x_t{};",
                encoded(&current_definition.name)
            )?;
        }

        for current_definition in &records {
            self.declare_type(
                &Type::Custom(current_definition.name.clone()),
                &mut declarations,
            )?;
        }

        for name in &self.order {
            let signature = self.symbols.get(name).ok_or_else(|| {
                EmitError::internal(backend_messages::c_emission_registered_function_is_missing())
            })?;
            self.declare_type(signature.return_value, &mut declarations)?;
            for value_type in signature.parameters {
                self.declare_type(value_type, &mut declarations)?;
            }

            for local in signature.body.locals.values() {
                self.declare_type(&local.value_type, &mut declarations)?;
            }
        }

        let mut throwing = BTreeSet::new();

        for name in &self.order {
            let signature = self.symbols.get(name).ok_or_else(|| {
                EmitError::internal(backend_messages::c_emission_registered_function_is_missing())
            })?;

            if signature.body.is_throws {
                let tag = throws_ctype(signature.return_value);

                if throwing.insert(tag.clone()) {
                    writeln!(
                        declarations.source,
                        "{tag}{{unsigned char ok;{} value;struct _x_str *error;}};",
                        if *signature.return_value == Type::Void {
                            "unsigned char".into()
                        } else {
                            ctype(signature.return_value)?
                        }
                    )?;
                }
            }
        }

        Ok(declarations.source)
    }

    pub(super) fn memoise_globals(&self) -> Result<String> {
        let mut memo_globals = String::new();

        for name in &self.order {
            let signature = self.symbols.get(name).ok_or_else(|| {
                EmitError::internal(backend_messages::c_emission_registered_function_is_missing())
            })?;

            if signature.body.is_memoised {
                memo_globals.push_str(&memoise::globals(&signature.symbol, signature)?);
            }
        }

        Ok(memo_globals)
    }

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
