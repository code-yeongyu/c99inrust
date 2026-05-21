use super::{Instruction, LoweredExpr, LoweringContext};
use crate::diagnostics::CompileResult;
use crate::parser::{BinaryOp, Expr, Statement, SwitchCase};
use std::collections::HashMap;

impl LoweringContext {
    pub(in crate::ir) fn lower_if(
        &mut self,
        condition: &Expr,
        then_branch: &Statement,
        else_branch: Option<&Statement>,
    ) -> CompileResult<()> {
        let else_label = self.fresh_label();
        let end_label = self.fresh_label();
        let condition = self.lower_expr(condition)?;
        self.instructions.push(Instruction::JumpIfZero {
            condition,
            label: else_label,
        });
        self.lower_branch(then_branch)?;
        if else_branch.is_some() {
            self.instructions
                .push(Instruction::Jump { label: end_label });
        }
        self.instructions
            .push(Instruction::Label { label: else_label });
        if let Some(statement) = else_branch {
            self.lower_branch(statement)?;
            self.instructions
                .push(Instruction::Label { label: end_label });
        }
        Ok(())
    }

    pub(in crate::ir) fn lower_while(
        &mut self,
        condition: &Expr,
        body: &Statement,
    ) -> CompileResult<()> {
        let start_label = self.fresh_label();
        let end_label = self.fresh_label();
        self.instructions
            .push(Instruction::Label { label: start_label });
        let condition = self.lower_expr(condition)?;
        self.instructions.push(Instruction::JumpIfZero {
            condition,
            label: end_label,
        });
        self.break_labels.push(end_label);
        self.continue_labels.push(start_label);
        let result = self.lower_branch(body);
        self.continue_labels.pop();
        self.break_labels.pop();
        result?;
        self.instructions
            .push(Instruction::Jump { label: start_label });
        self.instructions
            .push(Instruction::Label { label: end_label });
        Ok(())
    }

    pub(in crate::ir) fn lower_do_while(
        &mut self,
        body: &Statement,
        condition: &Expr,
    ) -> CompileResult<()> {
        let start_label = self.fresh_label();
        let continue_label = self.fresh_label();
        let end_label = self.fresh_label();
        self.instructions
            .push(Instruction::Label { label: start_label });
        self.break_labels.push(end_label);
        self.continue_labels.push(continue_label);
        let result = self.lower_branch(body);
        self.continue_labels.pop();
        self.break_labels.pop();
        result?;
        self.instructions.push(Instruction::Label {
            label: continue_label,
        });
        let condition = self.lower_expr(condition)?;
        self.instructions.push(Instruction::JumpIfZero {
            condition,
            label: end_label,
        });
        self.instructions
            .push(Instruction::Jump { label: start_label });
        self.instructions
            .push(Instruction::Label { label: end_label });
        Ok(())
    }

    pub(in crate::ir) fn lower_for(
        &mut self,
        initializer: Option<&Statement>,
        condition: Option<&Expr>,
        post: Option<&Statement>,
        body: &Statement,
    ) -> CompileResult<()> {
        self.scopes.push(HashMap::new());
        if let Some(statement) = initializer {
            self.lower_statement(statement)?;
        }
        let start_label = self.fresh_label();
        let continue_label = self.fresh_label();
        let end_label = self.fresh_label();
        self.instructions
            .push(Instruction::Label { label: start_label });
        if let Some(expr) = condition {
            let condition = self.lower_expr(expr)?;
            self.instructions.push(Instruction::JumpIfZero {
                condition,
                label: end_label,
            });
        }
        self.break_labels.push(end_label);
        self.continue_labels.push(continue_label);
        let result = self.lower_branch(body);
        self.continue_labels.pop();
        self.break_labels.pop();
        result?;
        self.instructions.push(Instruction::Label {
            label: continue_label,
        });
        if let Some(statement) = post {
            self.lower_statement(statement)?;
        }
        self.instructions
            .push(Instruction::Jump { label: start_label });
        self.instructions
            .push(Instruction::Label { label: end_label });
        self.pop_scope()
    }

    pub(in crate::ir) fn lower_switch(
        &mut self,
        condition: &Expr,
        cases: &[SwitchCase],
        default: &[Statement],
        default_position: Option<usize>,
    ) -> CompileResult<()> {
        self.scopes.push(HashMap::new());
        let end_label = self.fresh_label();
        let default_label = default_position.map(|_| self.fresh_label());
        let case_labels = (0..cases.len())
            .map(|_| self.fresh_label())
            .collect::<Vec<_>>();
        for (case, label) in cases.iter().zip(case_labels.iter().copied()) {
            let next_label = self.fresh_label();
            self.instructions.push(Instruction::JumpIfZero {
                condition: LoweredExpr::Binary {
                    op: BinaryOp::Equal,
                    left: Box::new(self.lower_expr(condition)?),
                    right: Box::new(self.lower_expr(&case.value)?),
                },
                label: next_label,
            });
            self.instructions.push(Instruction::Jump { label });
            self.instructions
                .push(Instruction::Label { label: next_label });
        }
        self.instructions.push(Instruction::Jump {
            label: default_label.unwrap_or(end_label),
        });
        self.break_labels.push(end_label);
        for (index, (case, label)) in cases.iter().zip(case_labels.iter().copied()).enumerate() {
            if default_position == Some(index)
                && let Some(label) = default_label
            {
                self.instructions.push(Instruction::Label { label });
                for statement in default {
                    self.lower_statement(statement)?;
                }
            }
            self.instructions.push(Instruction::Label { label });
            for statement in &case.statements {
                self.lower_statement(statement)?;
            }
        }
        if default_position == Some(cases.len())
            && let Some(label) = default_label
        {
            self.instructions.push(Instruction::Label { label });
            for statement in default {
                self.lower_statement(statement)?;
            }
        }
        self.break_labels.pop();
        self.instructions
            .push(Instruction::Label { label: end_label });
        self.pop_scope()
    }

    pub(in crate::ir) fn lower_branch(&mut self, statement: &Statement) -> CompileResult<()> {
        self.scopes.push(HashMap::new());
        self.lower_statement(statement)?;
        self.pop_scope()
    }
}
