use super::*;

impl Scalar<'_> {
    pub(in crate::scalar) fn function(&mut self) -> Attempt<Statement> {
        let mut pure = false;
        let mut memoised = false;
        let mut static_ = false;
        let mut multi = false;
        let mut throws = false;

        loop {
            match self.peek() {
                "pure" => pure = true,
                "memoise" => memoised = true,
                "static" => static_ = true,
                "multi" => multi = true,
                "throws" => throws = true,
                "unsafe" | "pub" => {}
                _ => break,
            }

            self.next();
        }

        self.need("fn")?;
        let name = self.name()?;
        self.need("(")?;
        let mut parameters = Vec::new();

        while self.peek() != ")" {
            let explicit_static = self.eat("static");
            let unique = self.eat("unique");

            if unique {
                return None;
            }

            let is_mut = self.eat("mut");
            let parameter_name = self.name()?;
            self.need(":")?;
            let value_type = self.value_type()?;
            parameters.push(Parameter {
                role: x_ast::ParameterRole::Argument,
                name: parameter_name,
                value_type,
                is_static: static_ || explicit_static,
                is_mut,
            });

            if !self.eat(",") {
                break;
            }
        }

        self.need(")")?;
        let return_type = if self.eat("->") {
            self.value_type()?
        } else {
            Type::Void
        };
        let body = if self.eat(";") {
            None
        } else {
            Some(Box::new(self.block()?))
        };

        Some(Statement::Function {
            name,
            generic_parameters: Vec::new(),
            parameters,
            return_type,
            body,
            is_pure: pure,
            is_memoised: memoised,
            is_static: static_,
            is_multi: multi,
            is_throws: throws,
            is_exported: false,
        })
    }

    pub(in crate::scalar) fn block(&mut self) -> Attempt<Vec<Statement>> {
        self.need("{")?;
        let mut statements = Vec::new();

        while self.peek() != "}" {
            if self.peek() == "" {
                return None;
            }

            statements.push(self.statement(true)?);
        }

        self.need("}")?;

        Some(statements)
    }

    pub(in crate::scalar) fn if_statement(&mut self) -> Attempt<Statement> {
        self.need("if")?;
        let condition = Box::new(self.expression(0)?);
        let then_block = self.block()?;
        let else_block = if self.eat("else") {
            Some(if self.peek() == "if" {
                vec![self.if_statement()?]
            } else {
                let body = self.block()?;
                self.statement_end = self.tokens.get(self.position.checked_sub(1)?)?.end;

                body
            })
        } else {
            self.statement_end = self.current().start;

            None
        };

        Some(Statement::If {
            condition,
            then_block,
            else_block,
        })
    }

    pub(in crate::scalar) fn import(&mut self) -> Attempt<Statement> {
        self.next();
        let mut parts = vec![self.name()?];

        while self.eat("::") {
            parts.push(self.name()?);
        }

        self.need(";")?;
        let (module, item) = crate::import::normalize_import_segments(parts).ok()?;

        Some(Statement::Import { module, item })
    }
}
