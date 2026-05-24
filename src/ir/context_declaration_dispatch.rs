use super::LoweringContext;
use crate::diagnostics::CompileResult;
use crate::parser::Statement;

impl LoweringContext {
    pub(in crate::ir) fn lower_declaration_like_statement(
        &mut self,
        statement: &Statement,
    ) -> Option<CompileResult<()>> {
        match statement {
            Statement::Declaration {
                is_static,
                scalar_type,
                name,
                referent,
                initializer,
            } => Some(self.lower_declaration(
                *is_static,
                *scalar_type,
                name,
                referent.clone(),
                initializer.as_ref(),
            )),
            Statement::LocalConstants(constants) => {
                for constant in constants {
                    self.constants.insert(constant.name.clone(), constant.value);
                }
                Some(Ok(()))
            }
            Statement::ExternGlobal(global) => Some(self.lower_extern_global(global)),
            statement => self.lower_local_declaration_statement(statement),
        }
    }

    fn lower_local_declaration_statement(
        &mut self,
        statement: &Statement,
    ) -> Option<CompileResult<()>> {
        match statement {
            Statement::LocalCharArray {
                name,
                length,
                is_unsigned,
                initializer,
            } => {
                Some(self.lower_local_char_array(name, *length, *is_unsigned, initializer.as_ref()))
            }
            Statement::LocalCharMatrix {
                name,
                rows,
                columns,
                initializer,
            } => Some(self.lower_local_char_matrix(name, *rows, *columns, initializer.as_deref())),
            Statement::LocalIntArray {
                name,
                length,
                initializer,
            } => Some(self.lower_local_int_array(name, *length, initializer.as_deref())),
            Statement::LocalIntMatrix {
                name,
                rows,
                columns,
                initializer,
            } => Some(self.lower_local_int_matrix(name, *rows, *columns, initializer.as_deref())),
            Statement::LocalShortArray {
                name,
                length,
                is_unsigned,
            } => Some(self.lower_local_short_array(name, *length, *is_unsigned)),
            Statement::LocalScalarArray {
                name,
                scalar_type,
                length,
                initializer,
            } => Some(self.lower_local_scalar_array(
                name,
                *scalar_type,
                *length,
                initializer.as_deref(),
            )),
            Statement::LocalPointerArray {
                name,
                length,
                referent,
                initializer,
            } => Some(self.lower_local_pointer_array(
                name,
                *length,
                referent.clone(),
                initializer.as_deref(),
            )),
            Statement::LocalStruct {
                name,
                struct_name,
                initializer,
            } => Some(self.lower_local_struct_object(name, struct_name, initializer.as_ref())),
            Statement::LocalStructArray {
                name,
                struct_name,
                length,
                initializer,
            } => Some(self.lower_local_struct_array(
                name,
                struct_name,
                *length,
                initializer.as_deref(),
            )),
            Statement::Empty
            | Statement::Block(_)
            | Statement::Declaration { .. }
            | Statement::DeclarationList(_)
            | Statement::ExpressionList(_)
            | Statement::Assignment { .. }
            | Statement::If { .. }
            | Statement::While { .. }
            | Statement::DoWhile { .. }
            | Statement::For { .. }
            | Statement::Switch { .. }
            | Statement::Expression(_)
            | Statement::Break
            | Statement::Continue
            | Statement::Label(_)
            | Statement::Goto(_)
            | Statement::Return(_)
            | Statement::LocalConstants(_)
            | Statement::ExternGlobal(_) => None,
        }
    }
}
