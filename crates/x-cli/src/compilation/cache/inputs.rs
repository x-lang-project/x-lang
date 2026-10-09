use super::*;

pub(super) fn hash_file(hash: &mut Sha256, path: &Path) -> Option<()> {
    update(hash, format!("{:?}", path.canonicalize().ok()?).as_bytes());
    update(hash, x_project::source::read_utf8(path).ok()?.as_bytes());

    Some(())
}

pub(super) fn workspace_files(hash: &mut Sha256, manifest_path: &Path) -> Option<()> {
    for ancestor in manifest_path.parent()?.ancestors() {
        let path = ancestor.join(MANIFEST_FILE_NAME);

        if !path.exists() {
            continue;
        }

        let manifest = Manifest::load(&path).ok()?;
        let Some(workspace) = manifest.workspace() else {
            continue;
        };

        hash_file(hash, &path)?;
        for member in &workspace.members {
            let path = x_utils::project::manifest_in(&ancestor.join(member))?;
            hash_file(hash, &path)?;
            let lock = path.parent()?.join(LOCK_FILE_NAME);

            if lock.exists() {
                hash_file(hash, &lock)?;
            }
        }

        break;
    }

    Some(())
}

pub(super) fn manifests(
    hash: &mut Sha256,
    path: &Path,
    seen: &mut BTreeSet<PathBuf>,
    depth: usize,
) -> Option<Manifest> {
    if depth > 64 {
        return None;
    }

    let path = path.canonicalize().ok()?;
    let manifest = Manifest::load(&path).ok()?;
    let link = manifest.link_options();

    if manifest.os_options().is_some()
        || manifest.build_options().freestanding
        || manifest.target().is_some_and(|type_value| {
            ProductTarget::parse(type_value) != Some(ProductTarget::WindowsMsvc)
        })
        || !link.libraries.is_empty()
        || !link.library_paths.is_empty()
        || !link.include_paths.is_empty()
    {
        return None;
    }

    if !seen.insert(path.clone()) {
        return Some(manifest);
    }

    ensure_manifest_lock(&path, manifest.source(), manifest.value()).ok()?;
    hash_file(hash, &path)?;
    if manifest.package().is_none() {
        return manifests(
            hash,
            &manifest.default_member_manifest().ok()?,
            seen,
            depth + 1,
        );
    }

    hash_file(hash, &path.parent()?.join(LOCK_FILE_NAME))?;
    for dependency in manifest.dependencies() {
        let path = if dependency.path.is_file() {
            dependency.path.clone()
        } else {
            dependency.path.join(MANIFEST_FILE_NAME)
        };

        manifests(hash, &path, seen, depth + 1)?;
    }

    Some(manifest)
}
