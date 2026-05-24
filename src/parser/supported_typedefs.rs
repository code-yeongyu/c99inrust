use super::ScalarType;

pub(super) fn supported_typedef_scalar(name: &str) -> Option<ScalarType> {
    match name {
        "Atom" | "Bool" | "Colormap" | "Cursor" | "Drawable" | "FILE" | "Font" | "GC"
        | "GameMission_t" | "GameMode_t" | "KeyCode" | "KeySym" | "Language_t" | "Pixmap"
        | "ShmSeg" | "Status" | "Time" | "VisualID" | "Window" | "XID" | "ammotype_t"
        | "angle_t" | "boolean" | "buttoncode_t" | "byte" | "card_t" | "cheat_t" | "command_t"
        | "evtype_t" | "fixed_t" | "gameaction_t" | "gamestate_t" | "key_t" | "lighttable_t"
        | "mobjflag_t" | "mobjtype_t" | "playerstate_t" | "powerduration_t" | "powertype_t"
        | "psprnum_t" | "skill_t" | "slopetype_t" | "spritenum_t" | "statenum_t"
        | "weapontype_t" => Some(ScalarType::Int),
        "va_list" => Some(ScalarType::VaList),
        _ => None,
    }
}
