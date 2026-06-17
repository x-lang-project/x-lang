use crate::{Finding, Source, masking};

impl Source {
    pub(crate) fn scope_at(&self, line: usize, column: usize) -> crate::scope::Scope {
        self.test_scopes.at(line, column)
    }

    pub(crate) fn is_test_file(&self) -> bool {
        self.test_scopes.is_all()
    }

    pub(crate) fn mark_test_file(&mut self) {
        self.test_scopes.mark_all();
    }

    pub(crate) fn ast_scope_at(&self, line: usize, column: usize) -> crate::scope::Scope {
        self.test_scopes.ast_at(line, column)
    }

    pub(crate) fn is_explicit_test_file(&self) -> bool {
        self.test_scopes.explicit_file()
    }

    pub(crate) fn clear_inferred_test_file(&mut self) {
        self.test_scopes.clear_inferred();
    }

    pub fn parse(path: impl Into<String>, text: impl Into<String>) -> syn::Result<Self> {
        let path = path.into();
        let text = text.into();
        let ast = syn::parse_file(&text)?;
        let masked = masking::mask(&text);

        Ok(Self {
            test_scopes: crate::scope::TestScopes::new(&path, &ast),
            path,
            text,
            masked,
            ast,
        })
    }

    pub fn find(&self, pattern: &str) -> Option<usize> {
        self.masked.find(pattern)
    }

    pub fn line_offset(&self, line: usize) -> usize {
        if line <= 1 {
            return 0;
        }

        self.text
            .match_indices('\n')
            .nth(line - 2)
            .map_or(self.text.len(), |(index, _)| index + 1)
    }

    pub fn offset(&self, span: proc_macro2::Span) -> usize {
        let start = span.start();

        self.position_offset(start.line, start.column)
    }

    pub fn position_offset(&self, line: usize, column: usize) -> usize {
        let line = self.line_offset(line);
        let tail = &self.text[line..];
        let column = tail
            .char_indices()
            .nth(column)
            .map_or(tail.len(), |(index, _)| index);

        line + column
    }

    pub fn emit(
        &self,
        output: &mut Vec<Finding>,
        rule: &str,
        offset: usize,
        message: impl Into<String>,
    ) {
        let mut offset = offset.min(self.text.len());

        while !self.text.is_char_boundary(offset) {
            offset -= 1;
        }

        let before = &self.text[..offset];
        let line = before
            .bytes()
            .filter(|second_value| *second_value == b'\n')
            .count()
            + 1;
        let start = before.rfind('\n').map_or(0, |index| index + 1);
        let column = self.text[start..offset].chars().count() + 1;
        let end = self.text[offset..]
            .find('\n')
            .map_or(self.text.len(), |index| offset + index);
        let evidence: String = self.text[start..end].trim().chars().take(240).collect();
        output.push(Finding {
            scope: self.scope_at(line, column),
            rule: rule.into(),
            path: self.path.clone(),
            line,
            column,
            message: message.into(),
            evidence,
        });
    }
}
