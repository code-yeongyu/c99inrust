use std::collections::HashMap;

use crate::diagnostics::{CompileError, CompileResult};

use super::definition::MacroDefinition;

mod parser;
mod token;

use parser::ConditionParser;
use token::condition_tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ConditionalFrame {
    parent_active: bool,
    current_active: bool,
    branch_state: BranchState,
}

impl ConditionalFrame {
    const fn new(parent_active: bool, enabled: bool) -> Self {
        let branch_state = if parent_active && enabled {
            BranchState::Taken
        } else {
            BranchState::Available
        };
        Self {
            parent_active,
            current_active: parent_active && enabled,
            branch_state,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BranchState {
    Available,
    Taken,
    ElseSeen,
}

pub(super) fn push_condition(condition_stack: &mut Vec<ConditionalFrame>, enabled: bool) {
    let parent_active = all_conditions_active(condition_stack);
    condition_stack.push(ConditionalFrame::new(parent_active, enabled));
}

pub(super) fn update_elif(
    condition_stack: &mut [ConditionalFrame],
    enabled: bool,
    line_number: usize,
) -> CompileResult<()> {
    let Some(last) = condition_stack.last_mut() else {
        return Err(CompileError::new("unexpected #elif").at(line_number, 1));
    };
    if last.branch_state == BranchState::ElseSeen {
        return Err(CompileError::new("#elif after #else").at(line_number, 1));
    }
    if last.branch_state == BranchState::Taken || !last.parent_active {
        last.current_active = false;
    } else {
        last.current_active = enabled;
        if enabled {
            last.branch_state = BranchState::Taken;
        }
    }
    Ok(())
}

pub(super) fn update_else(
    condition_stack: &mut [ConditionalFrame],
    line_number: usize,
) -> CompileResult<()> {
    let Some(last) = condition_stack.last_mut() else {
        return Err(CompileError::new("unexpected #else").at(line_number, 1));
    };
    if last.branch_state == BranchState::ElseSeen {
        return Err(CompileError::new("duplicate #else").at(line_number, 1));
    }
    let branch_taken = last.branch_state == BranchState::Taken;
    last.branch_state = BranchState::ElseSeen;
    last.current_active = last.parent_active && !branch_taken;
    Ok(())
}

pub(super) fn all_conditions_active(condition_stack: &[ConditionalFrame]) -> bool {
    condition_stack
        .iter()
        .all(|condition| condition.current_active)
}

pub(super) fn eval_condition(
    source: &str,
    macros: &HashMap<String, MacroDefinition>,
    line_number: usize,
) -> CompileResult<bool> {
    let tokens = condition_tokens(source, line_number)?;
    let mut parser = ConditionParser::new(tokens, macros, line_number);
    parser.expression()
}
