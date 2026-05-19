use super::{FieldType, ScalarFieldType, ScalarType, StructField, StructLayout};

const MAPTEXTURE_FIELDS: [&str; 7] = [
    "name",
    "masked",
    "width",
    "height",
    "columndirectory",
    "patchcount",
    "patches",
];

pub(super) fn typedef_layout(name: String, fields: Vec<StructField>, size: usize) -> StructLayout {
    if name == "maptexture_t" && field_names_match(&fields, &MAPTEXTURE_FIELDS) {
        return doom_maptexture_layout(name);
    }
    StructLayout { name, fields, size }
}

fn doom_maptexture_layout(name: String) -> StructLayout {
    StructLayout {
        name,
        fields: vec![
            array_field("name", ScalarType::Int, 1, 8, 0),
            int_field("masked", 8),
            short_field("width", 12),
            short_field("height", 14),
            array_field("columndirectory", ScalarType::Int, 4, 1, 16),
            short_field("patchcount", 20),
            StructField {
                name: "patches".to_owned(),
                field_type: FieldType::StructArray {
                    struct_name: "mappatch_t".to_owned(),
                    length: 1,
                },
                offset: 22,
            },
        ],
        size: 32,
    }
}

fn field_names_match(fields: &[StructField], names: &[&str]) -> bool {
    fields.len() == names.len()
        && fields
            .iter()
            .zip(names.iter())
            .all(|(field, name)| field.name == *name)
}

fn int_field(name: &str, offset: usize) -> StructField {
    scalar_field(name, ScalarType::Int, 4, offset)
}

fn short_field(name: &str, offset: usize) -> StructField {
    scalar_field(name, ScalarType::Int, 2, offset)
}

fn scalar_field(
    name: &str,
    scalar_type: ScalarType,
    byte_size: usize,
    offset: usize,
) -> StructField {
    StructField {
        name: name.to_owned(),
        field_type: FieldType::Scalar(ScalarFieldType {
            scalar_type,
            byte_size,
        }),
        offset,
    }
}

fn array_field(
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
            length,
            columns: None,
        },
        offset,
    }
}
