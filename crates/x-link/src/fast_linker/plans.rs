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
