use super::{
    AssignmentOperator, BinaryOp, Parser, TokenKind, token_is_assignment_operator,
    token_is_punctuator,
};

impl Parser<'_> {
    pub(super) fn current_identifier_starts_assignment(&self) -> bool {
        matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::Identifier(_))
        ) && self
            .tokens
            .get(self.index + 1)
            .is_some_and(token_is_assignment_operator)
    }

    pub(super) fn assignment_operator_at_current(&self) -> Option<AssignmentOperator> {
        let token = self.peek()?;
        if token_is_punctuator(token, "=") {
            Some(AssignmentOperator::Simple)
        } else if token_is_punctuator(token, "+=") {
            Some(AssignmentOperator::Compound(BinaryOp::Add))
        } else if token_is_punctuator(token, "-=") {
            Some(AssignmentOperator::Compound(BinaryOp::Sub))
        } else if token_is_punctuator(token, "*=") {
            Some(AssignmentOperator::Compound(BinaryOp::Mul))
        } else if token_is_punctuator(token, "/=") {
            Some(AssignmentOperator::Compound(BinaryOp::Div))
        } else if token_is_punctuator(token, "%=") {
            Some(AssignmentOperator::Compound(BinaryOp::Mod))
        } else if token_is_punctuator(token, "<<=") {
            Some(AssignmentOperator::Compound(BinaryOp::ShiftLeft))
        } else if token_is_punctuator(token, ">>=") {
            Some(AssignmentOperator::Compound(BinaryOp::ShiftRight))
        } else if token_is_punctuator(token, "&=") {
            Some(AssignmentOperator::Compound(BinaryOp::BitAnd))
        } else if token_is_punctuator(token, "^=") {
            Some(AssignmentOperator::Compound(BinaryOp::BitXor))
        } else if token_is_punctuator(token, "|=") {
            Some(AssignmentOperator::Compound(BinaryOp::BitOr))
        } else {
            None
        }
    }
}
