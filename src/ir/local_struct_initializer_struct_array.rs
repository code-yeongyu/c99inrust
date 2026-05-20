use super::{LoweringContext, StructAddress};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{FieldType, LocalStructInitializerValue};

impl LoweringContext {
    pub(in crate::ir) fn push_local_struct_array_initializer(
        &mut self,
        target: &StructAddress,
        offset: usize,
        field: &FieldType,
        values: &[LocalStructInitializerValue],
        value_index: &mut usize,
    ) -> CompileResult<()> {
        let FieldType::StructArray {
            struct_name,
            length,
        } = field
        else {
            return Err(CompileError::new("expected local struct-array field"));
        };
        let Some(value) = values.get(*value_index) else {
            return Ok(());
        };
        if let LocalStructInitializerValue::Nested(elements) = value {
            *value_index += 1;
            return self.push_braced_local_struct_array(
                target,
                offset,
                struct_name,
                *length,
                elements,
            );
        }
        self.push_unbraced_local_struct_array(
            target,
            offset,
            struct_name,
            *length,
            values,
            value_index,
        )
    }

    fn push_braced_local_struct_array(
        &mut self,
        target: &StructAddress,
        offset: usize,
        struct_name: &str,
        length: usize,
        elements: &[LocalStructInitializerValue],
    ) -> CompileResult<()> {
        if elements.len() > length {
            return Err(CompileError::new(
                "too many local struct-array initializer values",
            ));
        }
        let element_size = self.struct_layout(struct_name)?.size;
        for (index, element) in elements.iter().enumerate() {
            let nested_target =
                local_struct_array_element(target, offset, struct_name, element_size, index)?;
            self.push_single_struct_array_element(&nested_target, element)?;
        }
        Ok(())
    }

    fn push_unbraced_local_struct_array(
        &mut self,
        target: &StructAddress,
        offset: usize,
        struct_name: &str,
        length: usize,
        values: &[LocalStructInitializerValue],
        value_index: &mut usize,
    ) -> CompileResult<()> {
        let element_size = self.struct_layout(struct_name)?.size;
        for index in 0..length {
            if *value_index >= values.len() {
                return Ok(());
            }
            let nested_target =
                local_struct_array_element(target, offset, struct_name, element_size, index)?;
            let before = *value_index;
            self.push_local_struct_initializer_values(&nested_target, values, value_index)?;
            if *value_index == before {
                return Ok(());
            }
        }
        Ok(())
    }

    fn push_single_struct_array_element(
        &mut self,
        target: &StructAddress,
        element: &LocalStructInitializerValue,
    ) -> CompileResult<()> {
        let LocalStructInitializerValue::Nested(values) = element else {
            return Err(CompileError::new(
                "local struct-array initializers require nested values",
            ));
        };
        let mut nested_index = 0usize;
        self.push_local_struct_initializer_values(target, values, &mut nested_index)?;
        if nested_index == values.len() {
            Ok(())
        } else {
            Err(CompileError::new(
                "too many nested local struct-array initializer values",
            ))
        }
    }
}

fn local_struct_array_element(
    target: &StructAddress,
    offset: usize,
    struct_name: &str,
    element_size: usize,
    index: usize,
) -> CompileResult<StructAddress> {
    let offset = index
        .checked_mul(element_size)
        .and_then(|element_offset| offset.checked_add(element_offset))
        .ok_or_else(|| CompileError::new("local struct-array initializer offset overflow"))?;
    Ok(StructAddress {
        pointer: target.pointer.clone(),
        offset,
        struct_name: struct_name.to_owned(),
    })
}
