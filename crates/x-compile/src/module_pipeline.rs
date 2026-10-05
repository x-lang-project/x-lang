use std::path::Path;

use crate::SourceCatalog;
use crate::frontend;
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

pub fn compile_module_graph_to_mir(
    resolver: &ModuleResolver,
    entry_source: &str,
    entry_path: &Path,
    options: ModuleCompileOptions,
) -> Result<MirProgram, CompilationFailure> {
    let mut program = ModuleProgram::default();

    for name in resolver.order() {
        let node = resolver
            .modules()
            .get(name)
            .ok_or_else(|| contract_messages::resolved_module_missing(name))?;
        program.append(node, node.parse_for_compilation()?)?;
    }

    program.lower(entry_source, entry_path, options)
}

pub fn compile_owned_module_graph_to_mir(
    resolver: ModuleResolver,
    entry_source: &str,
    entry_path: &Path,
    options: ModuleCompileOptions,
) -> Result<MirProgram, CompilationFailure> {
    let mut program = ModuleProgram::default();

    for module in resolver.into_compilation_modules()? {
        program.append(&module.node, module.parsed)?;
    }

    program.lower(entry_source, entry_path, options)
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

    fn lower(
        mut self,
        entry_source: &str,
        entry_path: &Path,
        options: ModuleCompileOptions,
    ) -> Result<MirProgram, CompilationFailure> {
        let mut declarations = Vec::new();
        let mut functions = Vec::new();

        for item in self.items {
            match item {
                Statement::Function { .. } => functions.push(item),
                _ => declarations.push(item),
            }
        }

        declarations.extend(functions);

        frontend::check_and_lower(
            Program {
                statements: declarations,
            },
            frontend::FrontendOptions {
                no_borrow_check: options.no_borrow_check,
                contract_phase: contract_messages::ContractValidationPhase::ManifestLowering,
                ..Default::default()
            },
            frontend::SourceContext {
                input: frontend::SourceInput {
                    text: entry_source,
                    path: entry_path,
                },
                catalog: Some(&mut self.sources),
            },
        )
        .map(|checked| checked.mir)
    }
}
