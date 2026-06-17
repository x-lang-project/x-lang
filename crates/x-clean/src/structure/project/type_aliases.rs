mod collection;
mod index;

use syn::visit::Visit;

use super::module_path;
use crate::{Finding, Source};
use index::AliasIndex;

pub(super) fn analyse(sources: &[Source], findings: &mut Vec<Finding>) {
    let mut index = AliasIndex::default();

    for (source, data) in sources.iter().enumerate() {
        let scope = index.module_scope(module_path::source_module(&data.path));
        collection::Collector {
            index: &mut index,
            source,
            data,
            scope,
        }
        .visit_file(&data.ast);
    }

    for (alias, depth) in index.chains() {
        if depth >= 3 {
            sources[alias.source].emit(
                findings,
                "structure-type-alias-chain",
                alias.offset,
                format!("Local type alias follows {depth} named aliases"),
            );
        }
    }
}
