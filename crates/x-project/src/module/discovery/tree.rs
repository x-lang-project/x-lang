use super::*;

struct PendingModule {
    name: String,
    path: PathBuf,
    depth: usize,
}

pub(in crate::module) struct Discovery {
    pub(in crate::module) modules: HashMap<String, ModuleNode>,
    pub(in crate::module) parsed_modules: HashMap<String, x_parser::ParsedModule>,
    pub(in crate::module) total_source_bytes: usize,
}

impl ModuleResolver {
    pub(in crate::module) fn discover_tree(
        &self,
        current_name: &str,
        current_path: &Path,
    ) -> Result<Discovery, CompilationFailure> {
        let mut discovered = HashMap::new();
        let mut parsed_modules = HashMap::new();
        let mut total_source_bytes = self.total_source_bytes;
        let mut pending = vec![PendingModule {
            name: current_name.to_owned(),
            path: current_path.to_owned(),
            depth: 0,
        }];

        while let Some(PendingModule {
            name: module_name,
            path: module_path,
            depth,
        }) = pending.pop()
        {
            if depth > MAX_MODULE_DEPTH {
                return Err(ProjectError::module_depth_limit(
                    module_name,
                    MAX_MODULE_DEPTH,
                ));
            }

            if let Some(existing) = discovered
                .get(&module_name)
                .or_else(|| self.modules.get(&module_name))
            {
                if existing.path != module_path {
                    return Err(ProjectError::module_identity_conflict(
                        existing.path.display(),
                        module_path.display(),
                        module_name,
                    ));
                }

                continue;
            }

            if self.modules.len() + discovered.len() >= MAX_PROJECT_MODULES {
                return Err(ProjectError::module_count_limit(MAX_PROJECT_MODULES));
            }

            let source = read_module_source(&module_name, &module_path)?;
            total_source_bytes = total_source_bytes
                .checked_add(source.len())
                .ok_or_else(ProjectError::source_accounting_overflow)?;
            if total_source_bytes > MAX_PROJECT_SOURCE_BYTES {
                return Err(ProjectError::project_source_limit(MAX_PROJECT_SOURCE_BYTES));
            }

            let (mut parsed, imports) = parse_module_declarations(&source, &module_path)?;
            let submodules = if self.retain_parsed_modules {
                parsed.submodules.clone()
            } else {
                std::mem::take(&mut parsed.submodules)
            };

            let children = resolve_submodule_paths(&module_name, &module_path, &submodules)?;

            if self.retain_parsed_modules {
                parsed_modules.insert(module_name.clone(), parsed);
            }

            discovered.insert(
                module_name.clone(),
                ModuleNode {
                    name: module_name,
                    path: module_path,
                    source,
                    submodules,
                    imports,
                },
            );
            pending.extend(
                children
                    .into_iter()
                    .rev()
                    .map(|(name, path)| PendingModule {
                        name,
                        path,
                        depth: depth + 1,
                    }),
            );
        }

        Ok(Discovery {
            modules: discovered,
            parsed_modules,
            total_source_bytes,
        })
    }
}
