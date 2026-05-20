use super::builtin_field::{
    array_field, pointer_field, scalar_field, sized_array_field, struct_field, typed_scalar_field,
};
use super::{DOOM_EXPAND_PIXEL_UNION, DOOM_NAME8_UNION, ScalarType, StructField, StructLayout};

pub(super) fn builtin_struct_layouts() -> Vec<StructLayout> {
    let mut layouts = x11_event_struct_layouts();
    layouts.extend(x11_video_struct_layouts());
    layouts.extend(system_v_builtin_struct_layouts());
    layouts.extend(libc_builtin_struct_layouts());
    layouts.push(doom_expand_pixel_union_layout());
    layouts.push(doom_name8_union_layout());
    layouts
}

fn x11_event_struct_layouts() -> Vec<StructLayout> {
    vec![
        builtin_struct_layout("XKeyEvent", vec![scalar_field("keycode", 84)], 96),
        builtin_struct_layout(
            "XButtonEvent",
            vec![
                scalar_field("x", 64),
                scalar_field("y", 68),
                scalar_field("state", 80),
                scalar_field("button", 84),
            ],
            96,
        ),
        builtin_struct_layout(
            "XMotionEvent",
            vec![
                scalar_field("x", 64),
                scalar_field("y", 68),
                scalar_field("state", 80),
            ],
            96,
        ),
        builtin_struct_layout("XExposeEvent", vec![scalar_field("count", 56)], 64),
        builtin_struct_layout(
            "XEvent",
            vec![
                scalar_field("type", 0),
                struct_field("xkey", "XKeyEvent", 0),
                struct_field("xbutton", "XButtonEvent", 0),
                struct_field("xmotion", "XMotionEvent", 0),
                struct_field("xexpose", "XExposeEvent", 0),
            ],
            192,
        ),
    ]
}

fn x11_video_struct_layouts() -> Vec<StructLayout> {
    vec![
        builtin_struct_layout(
            "XVisualInfo",
            vec![
                pointer_field("visual", 0, Some("Visual")),
                scalar_field("depth", 20),
                scalar_field("class", 24),
            ],
            64,
        ),
        builtin_struct_layout(
            "XShmSegmentInfo",
            vec![
                scalar_field("shmid", 8),
                pointer_field("shmaddr", 16, Some("char")),
            ],
            32,
        ),
        builtin_struct_layout(
            "XImage",
            vec![
                scalar_field("height", 4),
                pointer_field("data", 16, Some("char")),
                scalar_field("bytes_per_line", 44),
            ],
            136,
        ),
        builtin_struct_layout(
            "XGCValues",
            vec![
                scalar_field("function", 0),
                scalar_field("graphics_exposures", 100),
            ],
            112,
        ),
        builtin_struct_layout(
            "XColor",
            vec![
                scalar_field("pixel", 0),
                scalar_field("red", 8),
                scalar_field("green", 10),
                scalar_field("blue", 12),
                scalar_field("flags", 14),
            ],
            16,
        ),
        builtin_struct_layout(
            "XSetWindowAttributes",
            vec![
                scalar_field("border_pixel", 24),
                scalar_field("event_mask", 72),
                scalar_field("colormap", 96),
            ],
            112,
        ),
    ]
}

fn system_v_builtin_struct_layouts() -> Vec<StructLayout> {
    vec![
        builtin_struct_layout("ipc_perm", vec![scalar_field("cuid", 12)], 48),
        builtin_struct_layout(
            "shmid_ds",
            vec![
                struct_field("shm_perm", "ipc_perm", 0),
                scalar_field("shm_segsz", 48),
                scalar_field("shm_cpid", 80),
                scalar_field("shm_nattch", 88),
            ],
            112,
        ),
    ]
}

fn libc_builtin_struct_layouts() -> Vec<StructLayout> {
    vec![
        builtin_struct_layout("in_addr", vec![scalar_field("s_addr", 0)], 4),
        builtin_struct_layout("sockaddr", vec![scalar_field("sa_family", 0)], 16),
        builtin_struct_layout(
            "sockaddr_in",
            vec![
                scalar_field("sin_family", 0),
                scalar_field("sin_port", 2),
                struct_field("sin_addr", "in_addr", 4),
            ],
            16,
        ),
        builtin_struct_layout(
            "timeval",
            vec![scalar_field("tv_sec", 0), scalar_field("tv_usec", 8)],
            16,
        ),
        builtin_struct_layout(
            "timezone",
            vec![
                scalar_field("tz_minuteswest", 0),
                scalar_field("tz_dsttime", 4),
            ],
            8,
        ),
        builtin_struct_layout(
            "stat",
            vec![scalar_field("st_size", 48), scalar_field("st_mtime", 96)],
            144,
        ),
        builtin_struct_layout(
            "hostent",
            vec![
                pointer_field("h_name", 0, Some("char")),
                pointer_field("h_aliases", 8, Some("*char")),
                scalar_field("h_addrtype", 16),
                scalar_field("h_length", 20),
                pointer_field("h_addr_list", 24, Some("*char")),
            ],
            32,
        ),
        builtin_struct_layout(
            "itimerval",
            vec![
                struct_field("it_interval", "timeval", 0),
                struct_field("it_value", "timeval", 16),
            ],
            32,
        ),
        builtin_struct_layout(
            "sigaction",
            vec![
                pointer_field("sa_handler", 0, None),
                scalar_field("sa_flags", 136),
            ],
            152,
        ),
    ]
}

fn doom_expand_pixel_union_layout() -> StructLayout {
    builtin_struct_layout(
        DOOM_EXPAND_PIXEL_UNION,
        vec![
            typed_scalar_field("d", ScalarType::Double, 0),
            array_field("u", ScalarType::Int, 2, 0),
        ],
        8,
    )
}

fn doom_name8_union_layout() -> StructLayout {
    builtin_struct_layout(
        DOOM_NAME8_UNION,
        vec![
            sized_array_field("s", ScalarType::Int, 1, 9, 0),
            array_field("x", ScalarType::Int, 2, 0),
        ],
        12,
    )
}

fn builtin_struct_layout(name: &str, fields: Vec<StructField>, size: usize) -> StructLayout {
    StructLayout {
        name: name.to_owned(),
        fields,
        size,
    }
}
