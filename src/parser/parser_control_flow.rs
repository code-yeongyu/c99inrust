use super::{
    CompileError, CompileResult, Keyword, Parser, Statement, SwitchCase, statement_from_expression,
};

impl Parser<'_> {
    pub(super) fn if_statement(&mut self) -> CompileResult<Statement> {
        self.expect_keyword(Keyword::If)?;
        self.expect_punctuator("(")?;
        let condition = self.expression()?;
        self.expect_punctuator(")")?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.check_keyword(Keyword::Else) {
            self.advance();
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    pub(super) fn while_statement(&mut self) -> CompileResult<Statement> {
        self.expect_keyword(Keyword::While)?;
        self.expect_punctuator("(")?;
        let condition = self.expression()?;
        self.expect_punctuator(")")?;
        let body = Box::new(self.statement()?);
        Ok(Statement::While { condition, body })
    }

    pub(super) fn do_while_statement(&mut self) -> CompileResult<Statement> {
        self.expect_keyword(Keyword::Do)?;
        let body = Box::new(self.statement()?);
        self.expect_keyword(Keyword::While)?;
        self.expect_punctuator("(")?;
        let condition = self.expression()?;
        self.expect_punctuator(")")?;
        self.expect_punctuator(";")?;
        Ok(Statement::DoWhile { body, condition })
    }

    pub(super) fn for_statement(&mut self) -> CompileResult<Statement> {
        self.expect_keyword(Keyword::For)?;
        self.expect_punctuator("(")?;
        let initializer = if self.check_punctuator(";") {
            self.advance();
            None
        } else if let Some(scalar_type) = self.declaration_type_at_current() {
            Some(Box::new(self.declaration_statement(scalar_type)?))
        } else {
            let initializer = self.comma_expression_statement()?;
            self.expect_punctuator(";")?;
            Some(Box::new(initializer))
        };
        let condition = if self.check_punctuator(";") {
            None
        } else {
            Some(self.expression()?)
        };
        self.expect_punctuator(";")?;
        let post = if self.check_punctuator(")") {
            None
        } else {
            Some(Box::new(self.comma_expression_statement()?))
        };
        self.expect_punctuator(")")?;
        let body = Box::new(self.statement()?);
        Ok(Statement::For {
            initializer,
            condition,
            post,
            body,
        })
    }

    pub(super) fn switch_statement(&mut self) -> CompileResult<Statement> {
        self.expect_keyword(Keyword::Switch)?;
        self.expect_punctuator("(")?;
        let condition = self.expression()?;
        self.expect_punctuator(")")?;
        self.expect_punctuator("{")?;
        let mut cases = Vec::new();
        let mut default = Vec::new();
        let mut saw_default = false;
        while !self.check_punctuator("}") {
            if self.check_keyword(Keyword::Case) {
                self.advance();
                let value = self.expression()?;
                self.expect_punctuator(":")?;
                let statements = self.switch_label_statements()?;
                cases.push(SwitchCase { value, statements });
                continue;
            }
            if self.check_keyword(Keyword::Default) {
                if saw_default {
                    return Err(CompileError::new("duplicate default label"));
                }
                saw_default = true;
                self.advance();
                self.expect_punctuator(":")?;
                default = self.switch_label_statements()?;
                continue;
            }
            return self.expected("switch case label");
        }
        self.expect_punctuator("}")?;
        Ok(Statement::Switch {
            condition,
            cases,
            default,
        })
    }

    pub(super) fn switch_label_statements(&mut self) -> CompileResult<Vec<Statement>> {
        let mut statements = Vec::new();
        while !self.check_punctuator("}")
            && !self.check_keyword(Keyword::Case)
            && !self.check_keyword(Keyword::Default)
        {
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    pub(super) fn block_items(&mut self) -> CompileResult<Vec<Statement>> {
        self.expect_punctuator("{")?;
        let mut statements = Vec::new();
        while !self.check_punctuator("}") {
            statements.push(self.statement()?);
        }
        self.expect_punctuator("}")?;
        Ok(statements)
    }

    pub(super) fn expression_statement(
        &mut self,
        expect_semicolon: bool,
    ) -> CompileResult<Statement> {
        let expr = self.expression()?;
        if expect_semicolon {
            self.expect_punctuator(";")?;
        }
        Ok(statement_from_expression(expr))
    }

    pub(super) fn comma_expression_statement(&mut self) -> CompileResult<Statement> {
        let mut statements = vec![self.expression_statement(false)?];
        while self.check_punctuator(",") {
            self.advance();
            statements.push(self.expression_statement(false)?);
        }
        if statements.len() == 1 {
            Ok(statements.remove(0))
        } else {
            Ok(Statement::ExpressionList(statements))
        }
    }
}
