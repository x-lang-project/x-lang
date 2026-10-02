use std::path::Path;

use crate::SourceCatalog;

use crate::qualify_module;
use x_ast::Program;
use x_ast::Statement;
use x_diagnostics::CompilationFailure;
use x_diagnostics::contract_messages;
use x_mir::MirProgram;
use x_project::ModuleNode;
use x_project::ModuleResolver;

#[derive(Clone, Copy, Debug, Default)]
pub struct ModuleCompileOptions {
    pub no_borrow_check: bool,
}

#[derive(Default)]
struct ModuleProgram {
    items: Vec<Statement>,
    sources: SourceCatalog,
}

impl ModuleProgram {
    fn append(
        &mut self,
        node: &ModuleNode,
        parsed: x_parser::ParsedModule,
    ) -> Result<(), CompilationFailure> {
        let mut part_program = parsed.program;

        if node.name != "crate" {
            let namespace_prefix = node.name.strip_prefix("crate::").unwrap_or(&node.name);
            qualify_module(&mut part_program, namespace_prefix);
        }

        self.sources.register(
            &mut part_program,
            &node.path,
            &node.source,
            &parsed.item_spans,
        )?;

        self.items.extend(part_program.statements);

        Ok(())
    }
}
