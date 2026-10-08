use x_diagnostics::{CompilationFailure, DriverError};
use x_project::ModuleResolver;
use x_project::manifest::Manifest;

pub fn resolve_manifest_modules(
    manifest: &Manifest,
    resolver: &mut ModuleResolver,
) -> Result<(), CompilationFailure> {
    let package = manifest
        .package()
        .ok_or_else(|| DriverError::manifest_package_missing(manifest.path().display()))?;

    resolver.resolve_tree("crate", &package.entry)?;
    for dependency in manifest.discover_crates(&package.name)? {
        resolver.resolve_tree(&dependency.import_name, &dependency.entry_path)?;
    }

    Ok(())
}
