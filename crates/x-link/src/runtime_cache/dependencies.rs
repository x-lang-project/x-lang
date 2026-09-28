use super::*;

pub(super) fn stamp(path: &Path) -> io::Result<Stamp> {
    let metadata = fs::metadata(path)?;
    let identity = NativeToolchain::file_identity(path);

    if cfg!(windows) && identity.is_none() {
        return Err(io::Error::other(
            "runtime cache requires file journal identity",
        ));
    }

    Ok(Stamp {
        path: path.to_path_buf(),
        modified_nanoseconds: metadata
            .modified()?
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos(),
        len: metadata.len(),
        directory: metadata.is_dir(),
        identity,
    })
}

pub(super) fn directory_stamps(roots: &[PathBuf]) -> io::Result<Vec<Stamp>> {
    let mut paths = BTreeMap::new();

    for root in roots {
        // Toolset/SDK selection can change when a new sibling version is added.
        for ancestor in root.ancestors() {
            if ancestor.is_dir() {
                paths.insert(ancestor.to_path_buf(), stamp(ancestor)?);
            }
        }

        if !root.is_dir() {
            continue;
        }

        let mut pending = vec![root.clone()];

        while let Some(directory) = pending.pop() {
            paths.insert(directory.clone(), stamp(&directory)?);

            for entry in fs::read_dir(directory)? {
                let entry = entry?;

                if entry.file_type()?.is_dir() {
                    pending.push(entry.path());
                } else if entry.file_type()?.is_symlink() {
                    return Err(io::Error::other(
                        "runtime header search tree contains symlinks",
                    ));
                }
            }
        }
    }

    Ok(paths.into_values().collect())
}

pub(super) fn include_directories(plan: &str) -> Option<Vec<PathBuf>> {
    let command = plan.lines().find(|line| line.contains("\"-cc1\""))?;
    let arguments = quoted_arguments(command)?;
    let mut roots = arguments
        .windows(2)
        .filter(|pair| {
            matches!(
                pair.first().map(String::as_str),
                Some("-internal-isystem" | "-internal-externc-isystem")
            )
        })
        .filter_map(|pair| pair.get(1).map(PathBuf::from))
        .collect::<Vec<_>>();
    // Use -v because Clang's configured search paths need not follow the user profile.
    for line in plan.lines() {
        if let Some(directory) = line
            .strip_prefix("System configuration file directory: ")
            .or_else(|| line.strip_prefix("User configuration file directory: "))
        {
            roots.push(PathBuf::from(directory));
        }
    }

    (!roots.is_empty()).then_some(roots)
}

pub(super) fn quoted_arguments(line: &str) -> Option<Vec<String>> {
    let mut chars = line.chars().peekable();
    let mut arguments = Vec::new();

    while chars.peek().is_some() {
        while chars
            .peek()
            .is_some_and(|character| character.is_whitespace())
        {
            chars.next();
        }

        if chars.peek().is_none() {
            break;
        }

        if chars.next()? != '"' {
            return None;
        }

        let mut argument = String::new();

        loop {
            match chars.next()? {
                '"' => break,
                '\\' => argument.push(chars.next()?),
                character => argument.push(character),
            }
        }

        arguments.push(argument);
    }

    Some(arguments)
}

pub(super) fn dependency_paths(contents: &str) -> Option<Vec<PathBuf>> {
    let input = contents
        .strip_prefix("x-runtime:")?
        .replace("\\\r\n", "")
        .replace("\\\n", "");
    let mut chars = input.chars().peekable();
    let mut paths = Vec::new();
    let mut token = String::new();

    while let Some(character) = chars.next() {
        match character {
            '\\' if chars
                .peek()
                .is_some_and(|next| matches!(next, ' ' | '\t' | '\\' | '#' | ':')) =>
            {
                token.push(chars.next()?);
            }
            character if character.is_whitespace() => {
                if !token.is_empty() {
                    paths.push(PathBuf::from(std::mem::take(&mut token)));
                }
            }
            '$' if chars.peek() == Some(&'$') => {
                chars.next();
                token.push('$');
            }
            character => token.push(character),
        }
    }

    if !token.is_empty() {
        paths.push(PathBuf::from(token));
    }

    (!paths.is_empty()).then_some(paths)
}

pub(super) fn digest(bytes: &[u8]) -> String {
    hexadecimal(&Sha256::digest(bytes))
}

pub(super) fn hexadecimal(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
