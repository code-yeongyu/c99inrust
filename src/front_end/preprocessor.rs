use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::diagnostics::{CompileError, CompileResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessedUnit {
    pub source: String,
    pub included_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct Preprocessor {
    include_paths: Vec<PathBuf>,
    predefined: HashMap<String, MacroDefinition>,
}

impl Preprocessor {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_include_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.include_paths.push(path.into());
        self
    }

    #[must_use]
    pub fn with_define(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.predefined.insert(
            name.into(),
            MacroDefinition::Object {
                replacement: value.into(),
            },
        );
        self
    }

    /// Preprocesses a file and recursively resolves local includes.
    ///
    /// # Errors
    ///
    /// Returns an error when source or include files cannot be read, directives
    /// are malformed, or conditional preprocessing is unbalanced.
    pub fn preprocess_file(&self, path: &Path) -> CompileResult<PreprocessedUnit> {
        let mut macros = self.predefined.clone();
        let mut included_files = Vec::new();
        let source = fs::read_to_string(path)
            .map_err(|error| CompileError::new(format!("failed to read source: {error}")))?;
        let current_dir = path.parent().map(Path::to_path_buf);
        let current_file = path.to_string_lossy();
        let output = self.preprocess_source(
            &source,
            current_file.as_ref(),
            current_dir.as_deref(),
            &mut macros,
            &mut included_files,
        )?;
        Ok(PreprocessedUnit {
            source: output,
            included_files,
        })
    }

    /// Preprocesses source text without an on-disk current directory.
    ///
    /// # Errors
    ///
    /// Returns an error when directives are malformed or conditional
    /// preprocessing is unbalanced.
    pub fn preprocess_text(&self, name: &str, source: &str) -> CompileResult<PreprocessedUnit> {
        let mut macros = self.predefined.clone();
        let mut included_files = Vec::new();
        let output =
            self.preprocess_source(source, name, None, &mut macros, &mut included_files)?;
        Ok(PreprocessedUnit {
            source: output,
            included_files,
        })
    }

    fn preprocess_source(
        &self,
        source: &str,
        current_file: &str,
        current_dir: Option<&Path>,
        macros: &mut HashMap<String, MacroDefinition>,
        included_files: &mut Vec<PathBuf>,
    ) -> CompileResult<String> {
        let mut output = String::new();
        let mut condition_stack = Vec::new();
        let spliced = splice_lines(source);
        let uncommented = remove_comments(&spliced)?;
        for (line_index, raw_line) in uncommented.lines().enumerate() {
            let line_number = line_index + 1;
            let trimmed = raw_line.trim_start();
            if let Some(directive) = trimmed.strip_prefix('#') {
                let mut state = PreprocessState {
                    current_dir,
                    macros,
                    included_files,
                    condition_stack: &mut condition_stack,
                    output: &mut output,
                };
                self.handle_directive(directive.trim_start(), line_number, &mut state)?;
                continue;
            }
            if all_conditions_active(&condition_stack) {
                let expanded = expand_macros(raw_line, macros);
                output.push_str(&expand_builtin_macros(&expanded, current_file, line_number));
                output.push('\n');
            }
        }
        if !condition_stack.is_empty() {
            return Err(CompileError::new("unterminated conditional directive"));
        }
        Ok(output)
    }

    fn handle_directive(
        &self,
        directive: &str,
        line_number: usize,
        state: &mut PreprocessState<'_>,
    ) -> CompileResult<()> {
        if let Some(rest) = directive.strip_prefix("define") {
            if all_conditions_active(state.condition_stack) {
                let (name, definition) = parse_define(rest.trim(), line_number)?;
                state.macros.insert(name, definition);
            }
            return Ok(());
        }
        if let Some(rest) = directive.strip_prefix("include") {
            if all_conditions_active(state.condition_stack) {
                match parse_include(rest.trim(), line_number)? {
                    Include::Local(include_path) => {
                        match self.resolve_include(&include_path, state.current_dir) {
                            Ok(resolved) => {
                                let source = fs::read_to_string(&resolved).map_err(|error| {
                                    CompileError::new(format!("failed to read include: {error}"))
                                })?;
                                state.included_files.push(resolved.clone());
                                let include_dir = resolved.parent().map(Path::to_path_buf);
                                let include_file = resolved.to_string_lossy();
                                let included = self.preprocess_source(
                                    &source,
                                    include_file.as_ref(),
                                    include_dir.as_deref(),
                                    state.macros,
                                    state.included_files,
                                )?;
                                state.output.push_str(&included);
                            }
                            Err(_error) if can_fall_back_to_system_include(&include_path) => {
                                state.output.push_str("#include \"");
                                state.output.push_str(&include_path);
                                state.output.push_str("\"\n");
                            }
                            Err(error) => return Err(error),
                        }
                    }
                    Include::System(include_path) => {
                        define_builtin_system_macros(&include_path, state.macros);
                        state.output.push_str("#include <");
                        state.output.push_str(&include_path);
                        state.output.push_str(">\n");
                    }
                }
            }
            return Ok(());
        }
        if let Some(rest) = directive.strip_prefix("undef") {
            if all_conditions_active(state.condition_stack) {
                state.macros.remove(rest.trim());
            }
            return Ok(());
        }
        if let Some(rest) = directive.strip_prefix("ifdef") {
            push_condition(
                state.condition_stack,
                state.macros.contains_key(rest.trim()),
            );
            return Ok(());
        }
        if let Some(rest) = directive.strip_prefix("ifndef") {
            push_condition(
                state.condition_stack,
                !state.macros.contains_key(rest.trim()),
            );
            return Ok(());
        }
        if let Some(rest) = directive.strip_prefix("if") {
            let enabled = eval_condition(rest.trim(), state.macros, line_number)?;
            push_condition(state.condition_stack, enabled);
            return Ok(());
        }
        if let Some(rest) = directive.strip_prefix("elif") {
            update_elif(
                state.condition_stack,
                eval_condition(rest.trim(), state.macros, line_number)?,
                line_number,
            )?;
            return Ok(());
        }
        if directive.starts_with("else") {
            update_else(state.condition_stack, line_number)?;
            return Ok(());
        }
        if directive.starts_with("endif") {
            if state.condition_stack.pop().is_none() {
                return Err(CompileError::new("unexpected #endif").at(line_number, 1));
            }
            return Ok(());
        }
        if all_conditions_active(state.condition_stack) {
            return Err(
                CompileError::new(format!("unsupported directive #{directive}")).at(line_number, 1),
            );
        }
        Ok(())
    }

    fn resolve_include(
        &self,
        include_path: &str,
        current_dir: Option<&Path>,
    ) -> CompileResult<PathBuf> {
        if let Some(dir) = current_dir {
            let candidate = dir.join(include_path);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
        for dir in &self.include_paths {
            let candidate = dir.join(include_path);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
        Err(CompileError::new(format!(
            "include not found: {include_path}"
        )))
    }
}

fn define_builtin_system_macros(include_path: &str, macros: &mut HashMap<String, MacroDefinition>) {
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
        "netinet/in.h" => Some(&[("IPPORT_USERRESERVED", "5000")]),
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
        "unistd.h" => Some(&[("R_OK", "4")]),
        "signal.h" => Some(&[("SIGINT", "2")]),
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum MacroDefinition {
    Object {
        replacement: String,
    },
    Function {
        params: Vec<String>,
        replacement: String,
    },
}

impl MacroDefinition {
    fn condition_value(&self) -> bool {
        match self {
            Self::Object { replacement } if replacement.trim().is_empty() => true,
            Self::Object { replacement } => replacement.trim().parse::<i64>().unwrap_or(0) != 0,
            Self::Function { .. } => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ConditionalFrame {
    parent_active: bool,
    current_active: bool,
    branch_state: BranchState,
}

impl ConditionalFrame {
    const fn new(parent_active: bool, enabled: bool) -> Self {
        let branch_state = if parent_active && enabled {
            BranchState::Taken
        } else {
            BranchState::Available
        };
        Self {
            parent_active,
            current_active: parent_active && enabled,
            branch_state,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BranchState {
    Available,
    Taken,
    ElseSeen,
}

struct PreprocessState<'a> {
    current_dir: Option<&'a Path>,
    macros: &'a mut HashMap<String, MacroDefinition>,
    included_files: &'a mut Vec<PathBuf>,
    condition_stack: &'a mut Vec<ConditionalFrame>,
    output: &'a mut String,
}

enum Include {
    Local(String),
    System(String),
}

fn splice_lines(source: &str) -> String {
    source.replace("\\\r\n", "").replace("\\\n", "")
}

fn remove_comments(source: &str) -> CompileResult<String> {
    let chars = source.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;
    let mut line = 1usize;
    let mut column = 1usize;
    while index < chars.len() {
        let current = chars[index];
        if current == '"' || current == '\'' {
            copy_quoted_and_update_position(
                &chars,
                &mut index,
                &mut output,
                &mut line,
                &mut column,
            );
            continue;
        }
        if current == '/' && chars.get(index + 1) == Some(&'/') {
            output.push(' ');
            advance_position(current, &mut line, &mut column);
            index += 1;
            advance_position(chars[index], &mut line, &mut column);
            index += 1;
            while chars.get(index).is_some_and(|value| *value != '\n') {
                advance_position(chars[index], &mut line, &mut column);
                index += 1;
            }
            continue;
        }
        if current == '/' && chars.get(index + 1) == Some(&'*') {
            let start_line = line;
            let start_column = column;
            output.push(' ');
            advance_position(current, &mut line, &mut column);
            index += 1;
            advance_position(chars[index], &mut line, &mut column);
            index += 1;
            let mut closed = false;
            while index < chars.len() {
                if chars[index] == '*' && chars.get(index + 1) == Some(&'/') {
                    advance_position(chars[index], &mut line, &mut column);
                    index += 1;
                    advance_position(chars[index], &mut line, &mut column);
                    index += 1;
                    closed = true;
                    break;
                }
                if chars[index] == '\n' {
                    output.push('\n');
                }
                advance_position(chars[index], &mut line, &mut column);
                index += 1;
            }
            if !closed {
                return Err(
                    CompileError::new("unterminated block comment").at(start_line, start_column)
                );
            }
            continue;
        }
        output.push(current);
        advance_position(current, &mut line, &mut column);
        index += 1;
    }
    Ok(output)
}

fn copy_quoted_and_update_position(
    chars: &[char],
    index: &mut usize,
    output: &mut String,
    line: &mut usize,
    column: &mut usize,
) {
    let quote = chars[*index];
    output.push(quote);
    advance_position(quote, line, column);
    *index += 1;
    let mut escaped = false;
    while *index < chars.len() {
        let current = chars[*index];
        output.push(current);
        *index += 1;
        advance_position(current, line, column);
        if escaped {
            escaped = false;
            continue;
        }
        if current == '\\' {
            escaped = true;
            continue;
        }
        if current == quote {
            break;
        }
    }
}

const fn advance_position(value: char, line: &mut usize, column: &mut usize) {
    if value == '\n' {
        *line += 1;
        *column = 1;
    } else {
        *column += 1;
    }
}

fn parse_define(rest: &str, line: usize) -> CompileResult<(String, MacroDefinition)> {
    let mut chars = rest.char_indices().peekable();
    let Some((_, first)) = chars.next() else {
        return Err(CompileError::new("expected macro name").at(line, 1));
    };
    if !is_identifier_start(first) {
        return Err(CompileError::new("expected macro name").at(line, 1));
    }
    let mut name = String::from(first);
    let mut end = first.len_utf8();
    while let Some((index, current)) = chars.peek().copied() {
        if !is_identifier_continue(current) {
            break;
        }
        name.push(current);
        end = index + current.len_utf8();
        chars.next();
    }
    let after_name = &rest[end..];
    if let Some(after_open) = after_name.strip_prefix('(') {
        let Some(close_index) = after_open.find(')') else {
            return Err(CompileError::new("unterminated function-like macro params").at(line, 1));
        };
        let params_source = &after_open[..close_index];
        let params = if params_source.trim().is_empty() {
            Vec::new()
        } else {
            params_source
                .split(',')
                .map(str::trim)
                .map(str::to_string)
                .collect()
        };
        let replacement = after_open[close_index + 1..].trim().to_string();
        return Ok((
            name,
            MacroDefinition::Function {
                params,
                replacement,
            },
        ));
    }
    let replacement = after_name.trim().to_string();
    Ok((name, MacroDefinition::Object { replacement }))
}

fn parse_include(rest: &str, line: usize) -> CompileResult<Include> {
    if let Some(stripped) = rest
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    {
        return Ok(Include::Local(stripped.to_string()));
    }
    if let Some(stripped) = rest
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
    {
        return Ok(Include::System(stripped.to_string()));
    }
    Err(CompileError::new("expected quoted or system include").at(line, 1))
}

fn can_fall_back_to_system_include(include_path: &str) -> bool {
    !include_path.contains('/') && !include_path.contains('\\')
}

fn push_condition(condition_stack: &mut Vec<ConditionalFrame>, enabled: bool) {
    let parent_active = all_conditions_active(condition_stack);
    condition_stack.push(ConditionalFrame::new(parent_active, enabled));
}

fn update_elif(
    condition_stack: &mut [ConditionalFrame],
    enabled: bool,
    line_number: usize,
) -> CompileResult<()> {
    let Some(last) = condition_stack.last_mut() else {
        return Err(CompileError::new("unexpected #elif").at(line_number, 1));
    };
    if last.branch_state == BranchState::ElseSeen {
        return Err(CompileError::new("#elif after #else").at(line_number, 1));
    }
    if last.branch_state == BranchState::Taken || !last.parent_active {
        last.current_active = false;
    } else {
        last.current_active = enabled;
        if enabled {
            last.branch_state = BranchState::Taken;
        }
    }
    Ok(())
}

fn update_else(condition_stack: &mut [ConditionalFrame], line_number: usize) -> CompileResult<()> {
    let Some(last) = condition_stack.last_mut() else {
        return Err(CompileError::new("unexpected #else").at(line_number, 1));
    };
    if last.branch_state == BranchState::ElseSeen {
        return Err(CompileError::new("duplicate #else").at(line_number, 1));
    }
    let branch_taken = last.branch_state == BranchState::Taken;
    last.branch_state = BranchState::ElseSeen;
    last.current_active = last.parent_active && !branch_taken;
    Ok(())
}

fn all_conditions_active(condition_stack: &[ConditionalFrame]) -> bool {
    condition_stack
        .iter()
        .all(|condition| condition.current_active)
}

fn expand_macros(line: &str, macros: &HashMap<String, MacroDefinition>) -> String {
    let mut current = line.to_string();
    for _ in 0..16 {
        let next = expand_macros_once(&current, macros);
        if next == current {
            return next;
        }
        current = next;
    }
    current
}

fn expand_builtin_macros(line: &str, current_file: &str, line_number: usize) -> String {
    let chars = line.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;
    while index < chars.len() {
        let current = chars[index];
        if current == '"' || current == '\'' {
            copy_quoted(&chars, &mut index, &mut output);
            continue;
        }
        if is_identifier_start(current) {
            let identifier = read_identifier(&chars, &mut index);
            match identifier.as_str() {
                "__FILE__" => output.push_str(&c_string_literal(current_file)),
                "__LINE__" => output.push_str(&line_number.to_string()),
                _ => output.push_str(&identifier),
            }
            continue;
        }
        output.push(current);
        index += 1;
    }
    output
}

fn c_string_literal(value: &str) -> String {
    let mut literal = String::from("\"");
    for current in value.chars() {
        match current {
            '\\' => literal.push_str("\\\\"),
            '"' => literal.push_str("\\\""),
            '\n' => literal.push_str("\\n"),
            '\r' => literal.push_str("\\r"),
            '\t' => literal.push_str("\\t"),
            _ => literal.push(current),
        }
    }
    literal.push('"');
    literal
}

fn expand_macros_once(line: &str, macros: &HashMap<String, MacroDefinition>) -> String {
    let chars = line.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;
    while index < chars.len() {
        let current = chars[index];
        if current == '"' || current == '\'' {
            copy_quoted(&chars, &mut index, &mut output);
            continue;
        }
        if is_identifier_start(current) {
            let identifier_start = index;
            let identifier = read_identifier(&chars, &mut index);
            if let Some(definition) = macros.get(&identifier) {
                match definition {
                    MacroDefinition::Object { replacement } => output.push_str(replacement),
                    MacroDefinition::Function {
                        params,
                        replacement,
                    } => {
                        let mut probe = index;
                        while chars.get(probe).is_some_and(|value| value.is_whitespace()) {
                            probe += 1;
                        }
                        if chars.get(probe) == Some(&'(') {
                            index = probe + 1;
                            if let Some(args) = read_macro_args(&chars, &mut index) {
                                output.push_str(&replace_params(replacement, params, &args));
                            } else {
                                output.push_str(&line[identifier_start..]);
                                return output;
                            }
                        } else {
                            output.push_str(&identifier);
                        }
                    }
                }
                continue;
            }
            output.push_str(&identifier);
            continue;
        }
        output.push(current);
        index += 1;
    }
    output
}

fn read_macro_args(chars: &[char], index: &mut usize) -> Option<Vec<String>> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    while *index < chars.len() {
        let value = chars[*index];
        match value {
            '"' | '\'' => copy_quoted(chars, index, &mut current),
            '(' => {
                depth += 1;
                current.push(value);
                *index += 1;
            }
            ')' if depth == 0 => {
                args.push(current.trim().to_string());
                *index += 1;
                return Some(args);
            }
            ')' => {
                depth -= 1;
                current.push(value);
                *index += 1;
            }
            ',' if depth == 0 => {
                args.push(current.trim().to_string());
                current.clear();
                *index += 1;
            }
            _ => {
                current.push(value);
                *index += 1;
            }
        }
    }
    None
}

fn replace_params(replacement: &str, params: &[String], args: &[String]) -> String {
    let chars = replacement.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;
    while index < chars.len() {
        let current = chars[index];
        if current == '"' || current == '\'' {
            copy_quoted(&chars, &mut index, &mut output);
            continue;
        }
        if is_identifier_start(current) {
            let identifier = read_identifier(&chars, &mut index);
            if let Some(param_index) = params.iter().position(|param| param == &identifier) {
                if let Some(arg) = args.get(param_index) {
                    output.push_str(arg);
                }
            } else {
                output.push_str(&identifier);
            }
            continue;
        }
        output.push(current);
        index += 1;
    }
    output
}

fn copy_quoted(chars: &[char], index: &mut usize, output: &mut String) {
    let quote = chars[*index];
    let mut escaped = false;
    output.push(quote);
    *index += 1;
    while *index < chars.len() {
        let current = chars[*index];
        output.push(current);
        *index += 1;
        if escaped {
            escaped = false;
            continue;
        }
        if current == '\\' {
            escaped = true;
            continue;
        }
        if current == quote {
            break;
        }
    }
}

fn read_identifier(chars: &[char], index: &mut usize) -> String {
    let mut value = String::new();
    while chars
        .get(*index)
        .is_some_and(|current| is_identifier_continue(*current))
    {
        value.push(chars[*index]);
        *index += 1;
    }
    value
}

const fn is_identifier_start(value: char) -> bool {
    value.is_ascii_alphabetic() || value == '_'
}

const fn is_identifier_continue(value: char) -> bool {
    value.is_ascii_alphanumeric() || value == '_'
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ConditionToken {
    Ident(String),
    Integer(i64),
    Defined,
    Bang,
    AndAnd,
    OrOr,
    EqEq,
    NotEq,
    LParen,
    RParen,
    End,
}

fn eval_condition(
    source: &str,
    macros: &HashMap<String, MacroDefinition>,
    line_number: usize,
) -> CompileResult<bool> {
    let tokens = condition_tokens(source, line_number)?;
    let mut parser = ConditionParser {
        tokens,
        index: 0,
        macros,
        line_number,
    };
    parser.expression()
}

fn condition_tokens(source: &str, line_number: usize) -> CompileResult<Vec<ConditionToken>> {
    let chars = source.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut index = 0usize;
    while index < chars.len() {
        match chars[index] {
            value if value.is_whitespace() => index += 1,
            value if is_identifier_start(value) => {
                let ident = read_identifier(&chars, &mut index);
                if ident == "defined" {
                    tokens.push(ConditionToken::Defined);
                } else {
                    tokens.push(ConditionToken::Ident(ident));
                }
            }
            value if value.is_ascii_digit() => {
                tokens.push(read_condition_integer(&chars, &mut index, line_number)?);
            }
            '/' if chars.get(index + 1) == Some(&'/') => break,
            '/' if chars.get(index + 1) == Some(&'*') => {
                index += 2;
                while index + 1 < chars.len() && !(chars[index] == '*' && chars[index + 1] == '/') {
                    index += 1;
                }
                if index + 1 < chars.len() {
                    index += 2;
                }
            }
            '!' if chars.get(index + 1) == Some(&'=') => {
                tokens.push(ConditionToken::NotEq);
                index += 2;
            }
            '!' => {
                tokens.push(ConditionToken::Bang);
                index += 1;
            }
            '&' if chars.get(index + 1) == Some(&'&') => {
                tokens.push(ConditionToken::AndAnd);
                index += 2;
            }
            '|' if chars.get(index + 1) == Some(&'|') => {
                tokens.push(ConditionToken::OrOr);
                index += 2;
            }
            '=' if chars.get(index + 1) == Some(&'=') => {
                tokens.push(ConditionToken::EqEq);
                index += 2;
            }
            '(' => {
                tokens.push(ConditionToken::LParen);
                index += 1;
            }
            ')' => {
                tokens.push(ConditionToken::RParen);
                index += 1;
            }
            _ => {
                return Err(
                    CompileError::new("unsupported #if expression token").at(line_number, 1)
                );
            }
        }
    }
    tokens.push(ConditionToken::End);
    Ok(tokens)
}

fn read_condition_integer(
    chars: &[char],
    index: &mut usize,
    line_number: usize,
) -> CompileResult<ConditionToken> {
    let start = *index;
    while chars.get(*index).is_some_and(char::is_ascii_digit) {
        *index += 1;
    }
    let value = chars[start..*index].iter().collect::<String>();
    let parsed = value
        .parse::<i64>()
        .map_err(|_| CompileError::new("integer literal is too large").at(line_number, 1))?;
    Ok(ConditionToken::Integer(parsed))
}

struct ConditionParser<'a> {
    tokens: Vec<ConditionToken>,
    index: usize,
    macros: &'a HashMap<String, MacroDefinition>,
    line_number: usize,
}

impl ConditionParser<'_> {
    fn expression(&mut self) -> CompileResult<bool> {
        self.or()
    }

    fn or(&mut self) -> CompileResult<bool> {
        let mut value = self.and()?;
        while self.matches(&ConditionToken::OrOr) {
            value = value || self.and()?;
        }
        Ok(value)
    }

    fn and(&mut self) -> CompileResult<bool> {
        let mut value = self.equality()?;
        while self.matches(&ConditionToken::AndAnd) {
            value = value && self.equality()?;
        }
        Ok(value)
    }

    fn equality(&mut self) -> CompileResult<bool> {
        let mut value = self.unary()?;
        loop {
            if self.matches(&ConditionToken::EqEq) {
                value = value == self.unary()?;
                continue;
            }
            if self.matches(&ConditionToken::NotEq) {
                value = value != self.unary()?;
                continue;
            }
            return Ok(value);
        }
    }

    fn unary(&mut self) -> CompileResult<bool> {
        if self.matches(&ConditionToken::Bang) {
            return Ok(!self.unary()?);
        }
        if self.matches(&ConditionToken::Defined) {
            return self.defined();
        }
        self.primary()
    }

    fn primary(&mut self) -> CompileResult<bool> {
        match self.peek() {
            ConditionToken::Integer(value) => {
                let value = *value != 0;
                self.index += 1;
                Ok(value)
            }
            ConditionToken::Ident(name) => {
                let value = self
                    .macros
                    .get(name)
                    .is_some_and(MacroDefinition::condition_value);
                self.index += 1;
                Ok(value)
            }
            ConditionToken::LParen => {
                self.index += 1;
                let value = self.expression()?;
                self.expect_token(&ConditionToken::RParen)?;
                Ok(value)
            }
            _ => Err(CompileError::new("expected #if expression").at(self.line_number, 1)),
        }
    }

    fn defined(&mut self) -> CompileResult<bool> {
        if self.matches(&ConditionToken::LParen) {
            let name = self.expect_ident()?;
            self.expect_token(&ConditionToken::RParen)?;
            return Ok(self.macros.contains_key(&name));
        }
        let name = self.expect_ident()?;
        Ok(self.macros.contains_key(&name))
    }

    fn expect_ident(&mut self) -> CompileResult<String> {
        let ConditionToken::Ident(name) = self.peek() else {
            return Err(CompileError::new("expected identifier").at(self.line_number, 1));
        };
        let name = name.clone();
        self.index += 1;
        Ok(name)
    }

    fn expect_token(&mut self, expected: &ConditionToken) -> CompileResult<()> {
        if self.matches(expected) {
            Ok(())
        } else {
            Err(CompileError::new("unexpected #if expression token").at(self.line_number, 1))
        }
    }

    fn matches(&mut self, expected: &ConditionToken) -> bool {
        if self.peek() == expected {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn peek(&self) -> &ConditionToken {
        &self.tokens[self.index]
    }
}
