use super::scalar_layout::{scalar_field_type, scalar_size_for_layout};
use super::{FieldType, ScalarType, StructField};

pub(super) fn scalar_field(name: &str, offset: usize) -> StructField {
    typed_scalar_field(name, ScalarType::Int, offset)
}

pub(super) fn typed_scalar_field(
    name: &str,
    scalar_type: ScalarType,
    offset: usize,
) -> StructField {
    StructField {
        name: name.to_owned(),
        field_type: FieldType::Scalar(scalar_field_type(&[], scalar_type)),
        offset,
    }
}

pub(super) fn array_field(
    name: &str,
    element_type: ScalarType,
    length: usize,
    offset: usize,
) -> StructField {
    sized_array_field(
        name,
        element_type,
        scalar_size_for_layout(element_type),
        length,
        offset,
    )
}

pub(super) fn sized_array_field(
    name: &str,
    element_type: ScalarType,
    element_size: usize,
    length: usize,
    offset: usize,
) -> StructField {
    StructField {
        name: name.to_owned(),
        field_type: FieldType::Array {
            element_type,
            element_size,
            element_unsigned: false,
            length,
            columns: None,
        },
        offset,
    }
}

pub(super) fn pointer_field(name: &str, offset: usize, referent: Option<&str>) -> StructField {
    StructField {
        name: name.to_owned(),
        field_type: FieldType::Pointer {
            referent: referent.map(ToOwned::to_owned),
        },
        offset,
    }
}

pub(super) fn struct_field(name: &str, struct_name: &str, offset: usize) -> StructField {
    StructField {
        name: name.to_owned(),
        field_type: FieldType::Struct(struct_name.to_owned()),
        offset,
    }
}
