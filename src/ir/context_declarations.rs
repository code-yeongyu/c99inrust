use super::{
    Instruction, LoweredExpr, LoweredGlobal, LoweredLValue, LoweringContext, insert_global_binding,
    local_char_array_initializer_values, local_char_matrix_initializer_values,
    local_pointer_array_byte_size, lower_extern_global_binding, scalar_size, static_local,
    zero_expr_for,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{
    Expr, Global, LocalCharArrayInitializer, LocalStructInitializer, LocalStructInitializerValue,
    ScalarType, Statement,
};
use std::collections::HashMap;

impl LoweringContext {
    pub(in crate::ir) fn lower_declaration(
        &mut self,
        is_static: bool,
        scalar_type: ScalarType,
        name: &str,
        referent: Option<String>,
        initializer: Option<&Expr>,
    ) -> CompileResult<()> {
        if is_static {
            return self.lower_static_declaration(scalar_type, name, referent, initializer);
        }
        let slot = self.declare_local(name, scalar_type, referent.clone())?;
        if scalar_type == ScalarType::Pointer
            && let Some(initializer @ Expr::ArrayCompoundLiteral { .. }) = initializer
        {
            return self.lower_array_compound_pointer_initializer(slot, initializer);
        }
        if scalar_type == ScalarType::Pointer
            && let Some(initializer @ Expr::AddressOf { .. }) = initializer
            && Self::is_scalar_compound_address(initializer)
        {
            return self.lower_scalar_compound_pointer_initializer(slot, initializer);
        }
        if scalar_type == ScalarType::Pointer
            && let Some(initializer) = initializer
            && Self::is_array_compound_element_address(initializer)
        {
            return self.lower_array_compound_element_pointer_initializer(slot, initializer);
        }
        if scalar_type == ScalarType::Pointer
            && let Some(initializer @ Expr::AddressOf { .. }) = initializer
            && Self::is_struct_compound_member_address(initializer)
        {
            return self.lower_struct_compound_member_pointer_initializer(slot, initializer);
        }
        if scalar_type == ScalarType::Pointer
            && let Some(initializer @ Expr::AddressOf { .. }) = initializer
            && Self::is_struct_compound_address(initializer)
        {
            return self.lower_struct_compound_pointer_initializer(slot, initializer);
        }
        let value = if scalar_type == ScalarType::Bool {
            initializer.map_or_else(
                || Ok(zero_expr_for(scalar_type)),
                |expr| self.lower_cast_expr(ScalarType::Bool, expr),
            )?
        } else {
            initializer.map_or_else(
                || Ok(zero_expr_for(scalar_type)),
                |expr| self.lower_expr(expr),
            )?
        };
        self.push_store(
            LoweredLValue::Local {
                slot,
                offset: self.local_offset(slot)?,
                scalar_type,
                referent,
            },
            value,
        )
    }

    pub(in crate::ir) fn lower_static_declaration(
        &mut self,
        scalar_type: ScalarType,
        name: &str,
        referent: Option<String>,
        initializer: Option<&Expr>,
    ) -> CompileResult<()> {
        if scalar_type == ScalarType::VaList {
            return Err(CompileError::new("static local does not support va_list"));
        }
        let initializer = static_local::scalar_initializer(
            scalar_type,
            referent.as_deref(),
            initializer,
            &self.constants,
            &self.structs,
            &self.global_bindings,
        )?;
        let global_name = self.declare_static_scalar(name, scalar_type, referent)?;
        self.static_globals.push(LoweredGlobal {
            name: global_name,
            is_static: true,
            initializer,
        });
        Ok(())
    }

    pub(in crate::ir) fn lower_local_char_array(
        &mut self,
        name: &str,
        length: usize,
        is_unsigned: bool,
        initializer: Option<&LocalCharArrayInitializer>,
    ) -> CompileResult<()> {
        let slot = self.declare_char_array(name, length, is_unsigned)?;
        if let Some(initializer) = initializer {
            self.instructions.push(Instruction::InitLocalBytes {
                offset: self.local_offset(slot)?,
                values: local_char_array_initializer_values(initializer, length)?,
            });
        }
        Ok(())
    }

    pub(in crate::ir) fn lower_local_char_matrix(
        &mut self,
        name: &str,
        rows: usize,
        columns: usize,
        initializer: Option<&[String]>,
    ) -> CompileResult<()> {
        let slot = self.declare_char_matrix(name, rows, columns)?;
        if let Some(values) = initializer {
            self.instructions.push(Instruction::InitLocalBytes {
                offset: self.local_offset(slot)?,
                values: local_char_matrix_initializer_values(values, rows, columns)?,
            });
        }
        Ok(())
    }

    pub(in crate::ir) fn lower_local_int_array(
        &mut self,
        name: &str,
        length: usize,
        initializer: Option<&[i32]>,
    ) -> CompileResult<()> {
        let slot = self.declare_int_array(name, length)?;
        if let Some(values) = initializer {
            self.instructions.push(Instruction::InitLocalInts {
                offset: self.local_offset(slot)?,
                values: values.to_vec(),
            });
        }
        Ok(())
    }

    pub(in crate::ir) fn lower_local_int_matrix(
        &mut self,
        name: &str,
        rows: usize,
        columns: usize,
        initializer: Option<&[i32]>,
    ) -> CompileResult<()> {
        let slot = self.declare_int_matrix(name, rows, columns)?;
        if let Some(values) = initializer {
            self.instructions.push(Instruction::InitLocalInts {
                offset: self.local_offset(slot)?,
                values: values.to_vec(),
            });
        }
        Ok(())
    }

    pub(in crate::ir) fn lower_local_pointer_array(
        &mut self,
        name: &str,
        length: usize,
        referent: Option<String>,
        initializer: Option<&[Expr]>,
    ) -> CompileResult<()> {
        let slot = self.declare_pointer_array(name, length, referent)?;
        if let Some(values) = initializer {
            if values.len() > length {
                return Err(CompileError::new(
                    "local pointer array initializer is too large",
                ));
            }
            let offset = self.local_offset(slot)?;
            let byte_size = local_pointer_array_byte_size(length)?;
            for (index, value) in values.iter().enumerate() {
                let index = i64::try_from(index)
                    .map_err(|_| CompileError::new("local pointer array index overflow"))?;
                let target = LoweredLValue::PointerSubscript {
                    pointer: Box::new(LoweredExpr::LocalAddress { offset, byte_size }),
                    index: Box::new(LoweredExpr::Integer(index)),
                    element_type: ScalarType::Pointer,
                    element_byte_size: scalar_size(ScalarType::Pointer),
                    element_unsigned: false,
                };
                let value = self.lower_expr(value)?;
                self.push_store(target, value)?;
            }
        }
        Ok(())
    }

    pub(in crate::ir) fn lower_extern_global(&mut self, global: &Global) -> CompileResult<()> {
        let Some(binding) = lower_extern_global_binding(&global.initializer, &self.structs)? else {
            return Err(CompileError::new(
                "internal error: non-extern block global declaration",
            ));
        };
        insert_global_binding(&mut self.global_bindings, &global.name, binding)
    }

    pub(in crate::ir) fn lower_local_struct_object(
        &mut self,
        name: &str,
        struct_name: &str,
        initializer: Option<&LocalStructInitializer>,
    ) -> CompileResult<()> {
        let slot = self.declare_struct_object(name, struct_name)?;
        if let Some(initializer) = initializer {
            self.lower_local_struct_initializer(name, struct_name, slot, initializer)?;
        }
        Ok(())
    }

    pub(in crate::ir) fn lower_local_struct_array(
        &mut self,
        name: &str,
        struct_name: &str,
        length: usize,
        initializer: Option<&[LocalStructInitializerValue]>,
    ) -> CompileResult<()> {
        let slot = self.declare_struct_array(name, struct_name, length)?;
        if let Some(values) = initializer {
            self.lower_local_struct_array_initializer(name, struct_name, slot, length, values)?;
        }
        Ok(())
    }

    pub(in crate::ir) fn lower_block(&mut self, statements: &[Statement]) -> CompileResult<()> {
        self.scopes.push(HashMap::new());
        for statement in statements {
            self.lower_statement(statement)?;
        }
        self.pop_scope()
    }
}
