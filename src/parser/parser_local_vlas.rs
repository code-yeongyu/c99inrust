use super::{
    BinaryOp, CompileResult, Expr, Parser, ScalarType, Statement,
    eval_integer_initializer_expr_with_constants,
};

impl Parser<'_> {
    pub(super) fn local_vla_array_declaration(
        &self,
        element: LocalVlaElement,
        name: &str,
        length: &Expr,
        has_second_dimension: bool,
    ) -> CompileResult<Option<Statement>> {
        if has_second_dimension
            || eval_integer_initializer_expr_with_constants(length, self.known_constants).is_ok()
        {
            return Ok(None);
        }
        if self.check_punctuator("=") {
            return self.expected("variable length array without initializer");
        }
        Ok(Some(Statement::Declaration {
            is_static: false,
            scalar_type: ScalarType::Pointer,
            name: name.to_owned(),
            referent: Some(element.referent().to_owned()),
            initializer: Some(vla_alloc_expr(length.clone(), element.byte_size())),
        }))
    }
}

#[derive(Clone, Copy)]
pub(super) enum LocalVlaElement {
    Int,
    Char { is_unsigned: bool },
}

impl LocalVlaElement {
    const fn referent(self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Char { is_unsigned: true } => "byte",
            Self::Char { is_unsigned: false } => "char",
        }
    }

    const fn byte_size(self) -> i64 {
        match self {
            Self::Int => 4,
            Self::Char { .. } => 1,
        }
    }
}

fn vla_alloc_expr(length: Expr, byte_size: i64) -> Expr {
    let size = if byte_size == 1 {
        length
    } else {
        Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(length),
            right: Box::new(Expr::Integer(byte_size)),
        }
    };
    Expr::Call {
        callee: "malloc".to_owned(),
        args: vec![size],
    }
}
