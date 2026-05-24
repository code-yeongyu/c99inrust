use super::{LoweredExpr, LoweredLValue, LoweringContext, local_short_array_byte_size};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::ScalarType;

impl LoweringContext {
    pub(in crate::ir) fn lower_local_short_array(
        &mut self,
        name: &str,
        length: usize,
        is_unsigned: bool,
        initializer: Option<&[i32]>,
    ) -> CompileResult<()> {
        let slot = self.declare_short_array(name, length, is_unsigned)?;
        let Some(values) = initializer else {
            return Ok(());
        };
        let offset = self.local_offset(slot)?;
        let byte_size = local_short_array_byte_size(length)?;
        for (index, value) in values.iter().enumerate() {
            let index = i64::try_from(index)
                .map_err(|_| CompileError::new("local short array index overflow"))?;
            let target = LoweredLValue::PointerSubscript {
                pointer: Box::new(LoweredExpr::LocalAddress { offset, byte_size }),
                index: Box::new(LoweredExpr::Integer(index)),
                element_type: ScalarType::Int,
                element_byte_size: 2,
                element_unsigned: is_unsigned,
            };
            self.push_store(target, LoweredExpr::Integer(i64::from(*value)))?;
        }
        Ok(())
    }
}
