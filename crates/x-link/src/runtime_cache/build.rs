use super::*;

pub(super) fn compile_flags(options: &LinkOptions<'_>) -> Vec<String> {
    let mut flags = vec![
        "-fopenmp".into(),
        x_utils::optimization::NativeOptimization::new(options.optimisation_level).argument(),
    ];

    if let Some(target) = options.target_triple {
        flags.push(format!("--target={target}"));
    }

    if options.gc_sections {
        flags.extend(["-ffunction-sections".into(), "-fdata-sections".into()]);
    }

    flags
}
