use super::{
    CompileError, CompileResult, Function, Parameter, Parser, Program, ReturnType, ScalarType,
    function_pointer_name, parameter_is_variadic, parameter_is_void, parameter_scalar_type,
    pointer_referent_type, previous_identifier_index, supported_return_type, token_identifier,
    top_level_function_open_paren, top_level_punctuator_index,
};

impl Parser<'_> {
    pub(super) fn program(&mut self) -> CompileResult<Program> {
        let mut functions = Vec::new();
        while !self.check_end() {
            functions.push(self.function()?);
        }
        Ok(Program {
            structs: Vec::new(),
            constants: Vec::new(),
            globals: Vec::new(),
            pointer_return_functions: Vec::new(),
            function_prototypes: Vec::new(),
            functions,
        })
    }

    pub(super) fn function(&mut self) -> CompileResult<Function> {
        let (return_type, name) = self.function_signature()?;
        let (mut parameters, is_variadic) = self.parameter_list()?;
        self.apply_old_style_parameter_declarations(&mut parameters)?;
        self.expect_punctuator("{")?;
        let mut statements = Vec::new();
        while !self.check_punctuator("}") {
            statements.push(self.statement()?);
        }
        self.expect_punctuator("}")?;
        Ok(Function {
            name,
            return_type,
            parameters,
            is_variadic,
            statements,
        })
    }

    pub(super) fn function_signature(&mut self) -> CompileResult<(ReturnType, String)> {
        let tokens = &self.tokens[self.index..];
        let Some(open_index) = top_level_function_open_paren(tokens) else {
            return self.expected("function parameter list");
        };
        let Some(name_index) = previous_identifier_index(tokens, open_index) else {
            return self.expected("function name");
        };
        let return_type = supported_return_type(&tokens[..name_index])
            .ok_or_else(|| CompileError::new("unsupported function return type"))?;
        let name = token_identifier(&tokens[name_index])
            .ok_or_else(|| CompileError::new("expected function name"))?
            .to_owned();
        self.index += open_index + 1;
        Ok((return_type, name))
    }

    pub(super) fn parameter_list(&mut self) -> CompileResult<(Vec<Parameter>, bool)> {
        let mut parameters = Vec::new();
        let mut is_variadic = false;
        let mut parameter_start = self.index;
        let mut depth = 0usize;
        while !self.check_end() {
            if self.check_punctuator("(") {
                depth += 1;
                self.advance();
                continue;
            }
            if self.check_punctuator(")") {
                if depth == 0 {
                    if parameter_start == self.index {
                        self.advance();
                        return Ok((parameters, is_variadic));
                    }
                    is_variadic |=
                        self.push_parameter(&mut parameters, parameter_start, self.index)?;
                    self.advance();
                    return Ok((parameters, is_variadic));
                }
                depth -= 1;
                self.advance();
                continue;
            }
            if depth == 0 && self.check_punctuator(",") {
                is_variadic |= self.push_parameter(&mut parameters, parameter_start, self.index)?;
                self.advance();
                parameter_start = self.index;
                continue;
            }
            self.advance();
        }
        Err(CompileError::new("unterminated function parameter list"))
    }

    pub(super) fn push_parameter(
        &self,
        parameters: &mut Vec<Parameter>,
        start: usize,
        end: usize,
    ) -> CompileResult<bool> {
        let tokens = &self.tokens[start..end];
        if parameter_is_void(tokens) {
            return Ok(false);
        }
        if parameter_is_variadic(tokens) {
            return Ok(true);
        }
        if let Some(name) = function_pointer_name(tokens) {
            parameters.push(Parameter {
                name,
                scalar_type: ScalarType::Pointer,
                referent: None,
            });
            return Ok(false);
        }
        if let Some(name) = implicit_int_parameter_name(tokens) {
            parameters.push(Parameter {
                name: name.to_owned(),
                scalar_type: ScalarType::Int,
                referent: None,
            });
            return Ok(false);
        }
        let Some(name) = parameter_name(tokens) else {
            return Err(CompileError::new("unsupported function parameter"));
        };
        let scalar_type = parameter_scalar_type(
            tokens,
            self.known_scalar_typedefs,
            self.known_pointer_typedefs,
        )
        .ok_or_else(|| CompileError::new("unsupported function parameter"))?;
        parameters.push(Parameter {
            name: name.to_owned(),
            scalar_type,
            referent: pointer_referent_type(tokens),
        });
        Ok(false)
    }

    fn apply_old_style_parameter_declarations(
        &mut self,
        parameters: &mut [Parameter],
    ) -> CompileResult<()> {
        while !self.check_end() && !self.check_punctuator("{") {
            let tokens = &self.tokens[self.index..];
            let Some(semicolon_index) = top_level_punctuator_index(tokens, ";") else {
                return Err(CompileError::new(
                    "unterminated old-style parameter declaration",
                ));
            };
            let declaration = &tokens[..semicolon_index];
            let Some(name) = parameter_name(declaration) else {
                return Err(CompileError::new(
                    "unsupported old-style parameter declaration",
                ));
            };
            let scalar_type = parameter_scalar_type(
                declaration,
                self.known_scalar_typedefs,
                self.known_pointer_typedefs,
            )
            .ok_or_else(|| CompileError::new("unsupported old-style parameter declaration"))?;
            if let Some(parameter) = parameters
                .iter_mut()
                .find(|parameter| parameter.name == name)
            {
                parameter.scalar_type = scalar_type;
                parameter.referent = pointer_referent_type(declaration);
            }
            self.index += semicolon_index + 1;
        }
        Ok(())
    }
}

fn parameter_name(tokens: &[crate::front_end::lexer::Token]) -> Option<&str> {
    let before = top_level_punctuator_index(tokens, "[").unwrap_or(tokens.len());
    tokens[..before].iter().rev().find_map(token_identifier)
}

fn implicit_int_parameter_name(tokens: &[crate::front_end::lexer::Token]) -> Option<&str> {
    (tokens.len() == 1)
        .then(|| token_identifier(&tokens[0]))
        .flatten()
}
