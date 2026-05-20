use std::collections::HashMap;

use super::definition::MacroDefinition;

pub(super) fn define_builtin_system_macros(
    include_path: &str,
    macros: &mut HashMap<String, MacroDefinition>,
) {
    let Some(definitions) = builtin_system_macro_definitions(include_path) else {
        return;
    };
    for (name, replacement) in definitions {
        macros.insert(
            (*name).to_string(),
            MacroDefinition::Object {
                replacement: (*replacement).to_string(),
            },
        );
    }
}

fn builtin_system_macro_definitions(
    include_path: &str,
) -> Option<&'static [(&'static str, &'static str)]> {
    match include_path {
        "values.h" => Some(&[
            ("MAXCHAR", "127"),
            ("MINCHAR", "(-128)"),
            ("MAXSHORT", "32767"),
            ("MINSHORT", "(-32768)"),
            ("MAXINT", "2147483647"),
            ("MININT", "(-2147483647 - 1)"),
            ("MAXLONG", "2147483647"),
            ("MINLONG", "(-2147483647 - 1)"),
        ]),
        "sys/socket.h" => Some(&[("PF_INET", "2"), ("SOCK_DGRAM", "2")]),
        "netinet/in.h" => Some(&[
            ("AF_INET", "2"),
            ("INADDR_ANY", "0"),
            ("IPPORT_USERRESERVED", "5000"),
            ("IPPROTO_UDP", "17"),
        ]),
        "errno.h" => Some(&[("EWOULDBLOCK", "11")]),
        "sys/ioctl.h" => Some(&[("FIONBIO", "21537")]),
        "sys/time.h" => Some(&[("ITIMER_REAL", "0")]),
        "signal.h" => Some(&[
            ("SIGINT", "2"),
            ("SIGALRM", "14"),
            ("SA_RESTART", "0x10000000"),
        ]),
        "stddef.h" | "stdlib.h" => Some(&[("NULL", "0")]),
        "fcntl.h" => Some(&[
            ("O_RDONLY", "0"),
            ("O_WRONLY", "1"),
            ("O_RDWR", "2"),
            ("O_CREAT", "64"),
            ("O_TRUNC", "512"),
            ("O_BINARY", "0"),
        ]),
        "stdio.h" => Some(&[
            ("NULL", "0"),
            ("SEEK_SET", "0"),
            ("SEEK_CUR", "1"),
            ("SEEK_END", "2"),
        ]),
        "unistd.h" => Some(&[("R_OK", "4"), ("X_OK", "1")]),
        "sys/ipc.h" | "sys/shm.h" => {
            Some(&[("IPC_RMID", "0"), ("IPC_STAT", "2"), ("IPC_CREAT", "512")])
        }
        "X11/Xlib.h" => Some(x11_xlib_builtin_macros()),
        "X11/keysym.h" => Some(x11_keysym_builtin_macros()),
        "X11/extensions/XShm.h" => Some(&[("ShmCompletion", "0")]),
        _ => None,
    }
}

const fn x11_xlib_builtin_macros() -> &'static [(&'static str, &'static str)] {
    &[
        ("KeyPress", "2"),
        ("KeyRelease", "3"),
        ("ButtonPress", "4"),
        ("ButtonRelease", "5"),
        ("MotionNotify", "6"),
        ("Expose", "12"),
        ("ConfigureNotify", "22"),
        ("Button1", "1"),
        ("Button2", "2"),
        ("Button3", "3"),
        ("KeyPressMask", "1"),
        ("KeyReleaseMask", "2"),
        ("ButtonPressMask", "4"),
        ("ButtonReleaseMask", "8"),
        ("PointerMotionMask", "64"),
        ("Button1Mask", "256"),
        ("Button2Mask", "512"),
        ("Button3Mask", "1024"),
        ("ExposureMask", "32768"),
        ("CWBorderPixel", "8"),
        ("CWEventMask", "2048"),
        ("CWColormap", "8192"),
        ("GCFunction", "1"),
        ("GCGraphicsExposures", "65536"),
        ("GXclear", "0"),
        ("False", "0"),
        ("True", "1"),
        ("None", "0"),
        ("CurrentTime", "0"),
        ("InputOutput", "1"),
        ("AllocAll", "1"),
        ("PseudoColor", "3"),
        ("GrabModeAsync", "1"),
        ("ZPixmap", "2"),
        ("DoRed", "1"),
        ("DoGreen", "2"),
        ("DoBlue", "4"),
        ("DefaultScreen", "XDefaultScreen"),
        ("RootWindow", "XRootWindow"),
    ]
}

const fn x11_keysym_builtin_macros() -> &'static [(&'static str, &'static str)] {
    &[
        ("XK_BackSpace", "65288"),
        ("XK_Tab", "65289"),
        ("XK_Return", "65293"),
        ("XK_Pause", "65299"),
        ("XK_Escape", "65307"),
        ("XK_Delete", "65535"),
        ("XK_space", "32"),
        ("XK_asciitilde", "126"),
        ("XK_Left", "65361"),
        ("XK_Up", "65362"),
        ("XK_Right", "65363"),
        ("XK_Down", "65364"),
        ("XK_F1", "65470"),
        ("XK_F2", "65471"),
        ("XK_F3", "65472"),
        ("XK_F4", "65473"),
        ("XK_F5", "65474"),
        ("XK_F6", "65475"),
        ("XK_F7", "65476"),
        ("XK_F8", "65477"),
        ("XK_F9", "65478"),
        ("XK_F10", "65479"),
        ("XK_F11", "65480"),
        ("XK_F12", "65481"),
        ("XK_KP_Equal", "65469"),
        ("XK_KP_Subtract", "65453"),
        ("XK_equal", "61"),
        ("XK_minus", "45"),
        ("XK_Shift_L", "65505"),
        ("XK_Shift_R", "65506"),
        ("XK_Control_L", "65507"),
        ("XK_Control_R", "65508"),
        ("XK_Meta_L", "65511"),
        ("XK_Meta_R", "65512"),
        ("XK_Alt_L", "65513"),
        ("XK_Alt_R", "65514"),
    ]
}
