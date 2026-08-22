use std::mem;

use std::collections::HashSet;

use crate::{TypeChecker, errors::TypeError};
use x_ast::{Parameter, Type};

const MAX_TYPE_RESOLUTION_DEPTH: usize = 64;

pub(super) struct FunctionContext {
    pub(super) name: String,
    pub(super) is_pure: bool,
    pub(super) scope_depth: usize,
    pub(super) immutable_parameters: HashSet<String>,
}

#[derive(Clone, Copy, Default)]
pub(super) struct EffectContext {
    in_anonymous_function: bool,
    handles_call: bool,
}

impl FunctionContext {
    pub(super) fn new(
        name: String,
        is_pure: bool,
        scope_depth: usize,
        parameters: &[Parameter],
    ) -> Self {
        Self {
            name,
            is_pure,
            scope_depth,
            immutable_parameters: parameters
                .iter()
                .filter(|parameter| !parameter.is_mut)
                .map(|parameter| parameter.name.clone())
                .collect(),
        }
    }
}

impl TypeChecker {
    pub(super) fn with_type_resolution<T>(
        &mut self,
        resource: crate::errors::TypeResource,
        check: impl FnOnce(&mut Self) -> Result<T, TypeError>,
    ) -> Result<T, TypeError> {
        if self.type_resolution_depth >= MAX_TYPE_RESOLUTION_DEPTH {
            return Err(TypeError::ResourceLimit {
                resource,
                maximum: MAX_TYPE_RESOLUTION_DEPTH,
            });
        }

        self.type_resolution_depth += 1;
        let result = check(self);
        self.type_resolution_depth -= 1;

        result
    }

    pub(super) fn in_anonymous_function(&self) -> bool {
        self.effects.in_anonymous_function
    }

    fn with_effect_context<T>(
        &mut self,
        context: EffectContext,
        check: impl FnOnce(&mut Self) -> Result<T, TypeError>,
    ) -> Result<T, TypeError> {
        let previous = mem::replace(&mut self.effects, context);
        let result = check(self);
        self.effects = previous;

        result
    }

    pub(super) fn with_handled_call_operand<T>(
        &mut self,
        check: impl FnOnce(&mut Self) -> Result<T, TypeError>,
    ) -> Result<T, TypeError> {
        let context = EffectContext {
            handles_call: true,
            ..self.effects
        };

        self.with_effect_context(context, check)
    }

    pub(super) fn with_call_handler<T>(
        &mut self,
        check: impl FnOnce(&mut Self, bool) -> Result<T, TypeError>,
    ) -> Result<T, TypeError> {
        let handled = self.effects.handles_call;
        let context = EffectContext {
            handles_call: false,
            ..self.effects
        };

        self.with_effect_context(context, |checker| check(checker, handled))
    }

    pub(super) fn with_anonymous_function<T>(
        &mut self,
        check: impl FnOnce(&mut Self) -> Result<T, TypeError>,
    ) -> Result<T, TypeError> {
        let context = EffectContext {
            in_anonymous_function: true,
            ..self.effects
        };

        self.with_effect_context(context, check)
    }

    pub(super) fn with_lexical_scope<T>(
        &mut self,
        check: impl FnOnce(&mut Self) -> Result<T, TypeError>,
    ) -> Result<T, TypeError> {
        self.symbols.enter_scope();
        let result = check(self);
        self.leave_purity_scope();
        self.symbols.exit_scope();

        result
    }

    pub(crate) fn current_function_name(&self) -> Option<&String> {
        self.function_contexts.last().map(|context| &context.name)
    }

    pub(super) fn with_named_function<T>(
        &mut self,
        name: String,
        is_pure: bool,
        parameters: &[Parameter],
        check: impl FnOnce(&mut Self) -> Result<T, TypeError>,
    ) -> Result<T, TypeError> {
        let return_type = self
            .symbols
            .get_function(&name)
            .map(|signature| signature.return_type.clone())
            .unwrap_or(Type::Unknown);

        self.with_lexical_scope(|checker| {
            let context = FunctionContext::new(
                name,
                is_pure,
                checker.symbols.variables.len().saturating_sub(1),
                parameters,
            );

            checker.function_contexts.push(context);
            let result = checker.with_effect_context(EffectContext::default(), |checker| {
                checker.with_return_type(return_type, check)
            });
            checker.function_contexts.pop();

            result
        })
    }

    // Anonymous functions inherit the enclosing namespace and effect policy.
    pub(super) fn with_return_type<T>(
        &mut self,
        expected: Type,
        check: impl FnOnce(&mut Self) -> Result<T, TypeError>,
    ) -> Result<T, TypeError> {
        self.return_types.push(expected);
        let result = check(self);
        self.return_types.pop();

        result
    }
}
