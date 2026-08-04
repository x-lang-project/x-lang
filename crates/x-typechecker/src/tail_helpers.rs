pub(crate) fn qualified_path_suffix_matches(registered: &str, requested: &str) -> bool {
    registered == requested
        || (!requested.contains("::")
            && !requested.is_empty()
            && registered
                .strip_suffix(requested)
                .is_some_and(|prefix| prefix.ends_with("::")))
}
