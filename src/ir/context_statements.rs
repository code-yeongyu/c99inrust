use super::{Instruction, LoweringContext};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, ReturnType, Statement};

impl LoweringContext {
    pub(in crate::ir) fn lower_statement(&mut self, statement: &Statement) -> CompileResult<()> {
        match statement {
            Statement::Empty => Ok(()),
            Statement::Block(statements) => self.lower_block(statements),
            Statement::Declaration {
                is_static,
                scalar_type,
                name,
                referent,
                initializer,
            } => self.lower_declaration(
                *is_static,
                *scalar_type,
                name,
                referent.clone(),
                initializer.as_ref(),
            ),
            Statement::LocalCharArray {
                name,
                length,
                is_unsigned,
                initializer,
            } => self.lower_local_char_array(name, *length, *is_unsigned, initializer.as_ref()),
            Statement::LocalCharMatrix {
                name,
                rows,
                columns,
                initializer,
            } => self.lower_local_char_matrix(name, *rows, *columns, initializer.as_deref()),
            Statement::LocalIntArray {
                name,
                length,
                initializer,
            } => self.lower_local_int_array(name, *length, initializer.as_deref()),
            Statement::LocalShortArray {
                name,
                length,
                is_unsigned,
            } => self.lower_local_short_array(name, *length, *is_unsigned),
            Statement::LocalPointerArray {
                name,
                length,
                initializer,
            } => self.lower_local_pointer_array(name, *length, initializer.as_deref()),
            Statement::LocalStruct { name, struct_name } => {
                self.lower_local_struct_object(name, struct_name)
            }
            Statement::LocalConstants(constants) => {
                for constant in constants {
                    self.constants.insert(constant.name.clone(), constant.value);
                }
                Ok(())
            }
            Statement::DeclarationList(declarations) | Statement::ExpressionList(declarations) => {
                self.lower_statement_list(declarations)
            }
            Statement::ExternGlobal(global) => self.lower_extern_global(global),
            Statement::Assignment { target, value } => self.lower_assignment(target, value),
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => self.lower_if(condition, then_branch, else_branch.as_deref()),
            Statement::While { condition, body } => self.lower_while(condition, body),
            Statement::DoWhile { body, condition } => self.lower_do_while(body, condition),
            Statement::For {
                initializer,
                condition,
                post,
                body,
            } => self.lower_for(
                initializer.as_deref(),
                condition.as_ref(),
                post.as_deref(),
                body,
            ),
            Statement::Switch {
                condition,
                cases,
                default,
            } => self.lower_switch(condition, cases, default),
            Statement::Expression(Expr::PostIncrement { target, decrement }) => {
                self.lower_post_increment_statement(target, *decrement)
            }
            Statement::Expression(expr) => self.lower_expression_statement(expr),
            Statement::Break => self.lower_break(),
            Statement::Continue => self.lower_continue(),
            Statement::Label(label) => {
                self.lower_label(label);
                Ok(())
            }
            Statement::Goto(label) => {
                self.lower_goto(label);
                Ok(())
            }
            Statement::Return(expr) => self.lower_return(expr.as_ref()),
        }
    }

    pub(in crate::ir) fn lower_statement_list(
        &mut self,
        statements: &[Statement],
    ) -> CompileResult<()> {
        for statement in statements {
            self.lower_statement(statement)?;
        }
        Ok(())
    }

    pub(in crate::ir) fn lower_expression_statement(&mut self, expr: &Expr) -> CompileResult<()> {
        let expr = self.lower_expr(expr)?;
        self.instructions.push(Instruction::Eval(expr));
        Ok(())
    }

    pub(in crate::ir) fn lower_break(&mut self) -> CompileResult<()> {
        let Some(label) = self.break_labels.last() else {
            return Err(CompileError::new("break statement outside loop"));
        };
        self.instructions.push(Instruction::Jump { label: *label });
        Ok(())
    }

    pub(in crate::ir) fn lower_continue(&mut self) -> CompileResult<()> {
        let Some(label) = self.continue_labels.last() else {
            return Err(CompileError::new("continue statement outside loop"));
        };
        self.instructions.push(Instruction::Jump { label: *label });
        Ok(())
    }

    pub(in crate::ir) fn lower_label(&mut self, name: &str) {
        let label = self.named_label(name);
        self.instructions.push(Instruction::Label { label });
    }

    pub(in crate::ir) fn lower_goto(&mut self, name: &str) {
        let label = self.named_label(name);
        self.instructions.push(Instruction::Jump { label });
    }

    pub(in crate::ir) fn lower_return(&mut self, expr: Option<&Expr>) -> CompileResult<()> {
        match (self.return_type, expr) {
            (ReturnType::Int | ReturnType::Pointer, Some(expr)) => {
                let value = self.lower_expr(expr)?;
                self.instructions.push(Instruction::Return(Some(value)));
            }
            (ReturnType::Int, None) => {
                return Err(CompileError::new("int function must return a value"));
            }
            (ReturnType::Pointer, None) => {
                return Err(CompileError::new("pointer function must return a value"));
            }
            (ReturnType::Void, Some(_)) => {
                return Err(CompileError::new("void function cannot return a value"));
            }
            (ReturnType::Void, None) => {
                self.instructions.push(Instruction::Return(None));
            }
        }
        self.has_return = true;
        Ok(())
    }
}
