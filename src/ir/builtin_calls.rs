pub(super) fn returns_pointer(name: &str) -> bool {
    matches!(
        name,
        "alloca"
            | "calloc"
            | "fdopen"
            | "fopen"
            | "getenv"
            | "gethostbyname"
            | "malloc"
            | "realloc"
            | "shmat"
            | "strerror"
            | "XCreateGC"
            | "XCreateImage"
            | "XGetVisualInfo"
            | "XOpenDisplay"
            | "XShmCreateImage"
    )
}
