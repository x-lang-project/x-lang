use super::*;

pub(super) fn plan_path(
    toolchain: &NativeToolchain,
    compiler: &Path,
    linker: &Path,
    arguments: &[String],
) -> Option<PathBuf> {
    let environment = env::vars_os().collect::<BTreeMap<_, _>>();
    let identity = (
        "host-link-plan-v1",
        compiler,
        toolchain.version(),
        toolchain.cache_identity()?,
        NativeToolchain::tool_identity(linker)?,
        NativeToolchain::discovery_fingerprint()?,
        arguments,
        environment,
        env::current_dir().ok()?,
    );
    let digest = Sha256::digest(format!("{identity:?}").as_bytes());
    let key = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let directory = env::var_os("X_RUNTIME_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir);

    Some(
        directory
            .join("x-lang-native-link-v1")
            .join(format!("{key}.json")),
    )
}

pub(super) fn read_plan(path: &Path, linker: &Path, input_count: usize) -> Option<LinkPlan> {
    let plan: LinkPlan = serde_json::from_slice(&fs::read(path).ok()?).ok()?;

    if !valid_arguments(&plan.arguments, input_count)
        || !plan.stamps.contains_key(linker)
        || plan
            .stamps
            .iter()
            .any(|(path, expected)| stamp(path).as_ref() != Some(expected))
    {
        return None;
    }

    Some(plan)
}

pub(super) fn derive_plan(
    toolchain: &NativeToolchain,
    request: &LinkRequest<'_>,
    linker: &Path,
) -> Option<LinkPlan> {
    let LinkRequest {
        arguments: current_arguments,
        inputs,
        output,
        options,
    } = *request;
    let mut command = toolchain.command();
    command.arg("-###").args(current_arguments);
    if options.reproducible {
        command.env("SOURCE_DATE_EPOCH", "0");
    }

    let output_text = command.output().ok()?;

    if !output_text.status.success() {
        return None;
    }

    let stderr = String::from_utf8(output_text.stderr).ok()?;
    let mut commands = stderr
        .lines()
        .filter(|line| line.trim_start().starts_with('"'));
    let tokens = parse_quoted_command(commands.next()?)?;

    if commands.next().is_some() {
        // A C compilation, response tool or other subprocess is not a link plan.
        return None;
    }

    let selected_path = Path::new(tokens.first()?);
    // Clang omits .exe from lld-link; canonicalisation does not add it.
    let selected_path = if selected_path.extension().is_none() {
        selected_path.with_extension("exe")
    } else {
        selected_path.to_path_buf()
    };

    let selected = selected_path.canonicalize().ok()?;

    if selected != linker.canonicalize().ok()? {
        return None;
    }

    let arguments = normalize(tokens.get(1..)?, inputs, output);

    if !valid_arguments(&arguments, inputs.len()) {
        return None;
    }

    let mut paths = BTreeSet::from([linker.to_path_buf()]);

    for path in arguments
        .iter()
        .filter_map(|argument| argument.strip_prefix("-libpath:"))
    {
        // New SDK/MSVC versions can appear above the selected directory.
        paths.extend(Path::new(path).ancestors().map(Path::to_path_buf));
    }

    let stamps = paths
        .into_iter()
        .filter_map(|path| stamp(&path).map(|value| (path, value)))
        .collect();

    Some(LinkPlan { arguments, stamps })
}

pub(super) fn valid_arguments(arguments: &[String], input_count: usize) -> bool {
    let expected_output = format!("-out:{OUTPUT_TOKEN}");

    if arguments
        .iter()
        .filter(|argument| *argument == &expected_output)
        .count()
        != 1
    {
        return false;
    }

    for index in 0..input_count {
        let expected = format!("{INPUT_PREFIX}{index}__");

        if !arguments.iter().any(|argument| argument == &expected) {
            return false;
        }
    }

    arguments.iter().all(|argument| {
        argument == &expected_output
            || argument == "-nologo"
            || argument.starts_with("-defaultlib:")
            || argument.starts_with("-nodefaultlib:")
            || argument.starts_with("-libpath:")
            || argument
                .strip_prefix(INPUT_PREFIX)
                .and_then(|index| index.strip_suffix("__"))
                .and_then(|index| index.parse::<usize>().ok())
                .is_some_and(|index| index < input_count)
    })
}

pub(crate) fn parse_quoted_command(line: &str) -> Option<Vec<String>> {
    let mut chars = line.chars().peekable();
    let mut arguments = Vec::new();

    while chars.peek().is_some() {
        while chars.peek().is_some_and(|ch| ch.is_whitespace()) {
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
                '\\' => match chars.next()? {
                    ch @ ('\\' | '"') => argument.push(ch),
                    _ => return None,
                },
                ch => argument.push(ch),
            }
        }

        if chars.peek().is_some_and(|ch| !ch.is_whitespace()) {
            return None;
        }

        arguments.push(argument);
    }

    Some(arguments)
}
