use super::*;

pub(super) fn explanations() -> &'static [ExplanationEntry] {
    const EXPLANATIONS: &[ExplanationEntry] = &[
        ExplanationEntry::standard(ErrorExplanation {
            code: "E0015",
            title: "unavailable standard-library module",
            summary: "The requested module is not included in the shipped X standard-library catalogue.",
            facts: &[
                "The diagnostic lists the available modules.",
                "The requested standard-library module is unavailable.",
            ],
            fixes: &[
                "Correct the module path or import one of the listed shipped modules.",
                "Use a project module for code that is not part of the shipped standard library.",
            ],
        }),
        ExplanationEntry::standard(ErrorExplanation {
            code: "E0001",
            title: "type mismatch",
            summary: "The expression type differs from the required type.",
            facts: &[
                "The required type can come from an annotation, return type, parameter or field.",
                "The found type is inferred from the expression at the highlighted location.",
            ],
            fixes: &[
                "Use an expression of the required type.",
                "Change the annotation or function signature if the expression is the intended type.",
                "Add an explicit cast or conversion when the conversion is intentional.",
            ],
        }),
        ExplanationEntry::standard(ErrorExplanation {
            code: "E0002",
            title: "purity constraint violation",
            summary: "A pure function calls an impure function or performs an observable operation.",
            facts: &[
                "Pure functions must have inspectable bodies and call only pure operations.",
                "A pure contract forbids operations that change externally observable state.",
            ],
            fixes: &[
                "Replace the operation with a pure alternative.",
                "Move the observable operation into a function without a pure contract.",
                "Remove pure from the caller if observable operations are intended.",
            ],
        }),
        ExplanationEntry::standard(ErrorExplanation {
            code: "E0003",
            title: "arity mismatch",
            summary: "The call has a different number of arguments from the signature.",
            facts: &[
                "The expected count comes from the resolved function signature.",
                "The received count comes from the call expression.",
            ],
            fixes: &[
                "Add the missing argument or remove the extra argument.",
                "Call an overload or helper with matching parameters.",
                "Change the function signature if the call represents the intended API.",
            ],
        }),
        ExplanationEntry::standard(ErrorExplanation {
            code: "E0004",
            title: "unknown field or method",
            summary: "The receiver type does not expose the requested field or method.",
            facts: &[
                "The receiver type is inferred from the expression before the dot.",
                "The requested member name is taken from the field or method access.",
            ],
            fixes: &[
                "Correct the field or method name.",
                "Use a value whose type has that field or method.",
                "Add the field or method to the relevant type if it belongs there.",
            ],
        }),
        ExplanationEntry::standard(ErrorExplanation {
            code: "E0005",
            title: "type checking failed",
            summary: "A type constraint failed; the diagnostic note identifies it.",
            facts: &[
                "This includes inference failures and expression requirements.",
                "The note identifies the failed requirement.",
            ],
            fixes: &[
                "Address the requirement described in the diagnostic note.",
                "For an inference failure, add explicit type arguments or annotations.",
            ],
        }),
        ExplanationEntry::unknown_identifier(ErrorExplanation {
            code: "E0006",
            title: "unknown identifier",
            summary: "A name was used before the compiler could find a matching local, function, import, or global.",
            facts: &[
                "The highlighted name is the unresolved use.",
                "Visible declarations and imports define the names the compiler can resolve.",
            ],
            fixes: &[
                "Declare the value before using it.",
                "Import the module or qualify the name with its module path.",
                "Correct a typo in the identifier.",
            ],
        }),
        ExplanationEntry::standard(ErrorExplanation {
            code: "B0002",
            title: "use of moved value",
            summary: "A non-copy value was used after being moved.",
            facts: &[
                "The move consumes the original binding.",
                "The later use requires the binding to still own a value.",
            ],
            fixes: &[
                "Borrow the value instead of moving it.",
                "Clone or otherwise copy the value before the move if duplication is intended.",
                "Reorder the code so the final use happens before the move.",
            ],
        }),
        ExplanationEntry::standard(ErrorExplanation {
            code: "P0001",
            title: "parse failed",
            summary: "The source could not be parsed as X syntax.",
            facts: &[
                "The highlight marks where the parser rejected the syntax.",
                "Nearby punctuation, braces, and missing expressions are common causes.",
            ],
            fixes: &[
                "Check the highlighted token and the line immediately before it.",
                "Add missing punctuation, closing delimiters, or the expression expected there.",
                "Simplify the surrounding block to isolate the syntax error.",
            ],
        }),
    ];

    EXPLANATIONS
}
