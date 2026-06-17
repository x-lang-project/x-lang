pub(crate) struct ReviewSink<'a> {
    pub(crate) source: &'a crate::Source,
    pub(crate) findings: &'a mut Vec<crate::Finding>,
}
impl ReviewSink<'_> {
    pub(crate) fn at_span(self, rule: &str, span: proc_macro2::Span, message: impl Into<String>) {
        self.source
            .emit(self.findings, rule, self.source.offset(span), message);
    }

    pub(crate) fn at_node(
        self,
        rule: &str,
        node: &impl syn::spanned::Spanned,
        message: impl Into<String>,
    ) {
        self.at_span(rule, node.span(), message);
    }
}
