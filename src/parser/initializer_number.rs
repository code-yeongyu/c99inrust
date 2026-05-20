use crate::diagnostics::{CompileError, CompileResult};

#[derive(Clone, Copy)]
pub(super) struct InitializerNumber {
    numerator: i128,
    denominator: i128,
}

impl InitializerNumber {
    pub(super) fn integer(value: i64) -> Self {
        Self {
            numerator: i128::from(value),
            denominator: 1,
        }
    }

    pub(super) fn new(numerator: i128, denominator: i128) -> CompileResult<Self> {
        if denominator == 0 {
            return Err(CompileError::new("integer initializer division by zero"));
        }
        if denominator < 0 {
            return Ok(Self {
                numerator: numerator
                    .checked_neg()
                    .ok_or_else(|| CompileError::new("integer initializer overflow"))?,
                denominator: denominator
                    .checked_neg()
                    .ok_or_else(|| CompileError::new("integer initializer overflow"))?,
            });
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }

    pub(super) fn decimal(value: &str) -> CompileResult<Self> {
        let Some((whole, fractional)) = value.split_once('.') else {
            return Err(CompileError::new("unsupported decimal initializer"));
        };
        let whole = if whole.is_empty() {
            0
        } else {
            whole
                .parse::<i128>()
                .map_err(|_| CompileError::new("decimal initializer is too large"))?
        };
        let fractional = if fractional.is_empty() {
            0
        } else {
            fractional
                .parse::<i128>()
                .map_err(|_| CompileError::new("decimal initializer is too large"))?
        };
        let mut denominator = 1i128;
        for _digit in value
            .split_once('.')
            .map_or("", |(_whole, fractional)| fractional)
            .chars()
        {
            denominator = denominator
                .checked_mul(10)
                .ok_or_else(|| CompileError::new("decimal initializer is too large"))?;
        }
        let numerator = whole
            .checked_mul(denominator)
            .and_then(|whole| whole.checked_add(fractional))
            .ok_or_else(|| CompileError::new("decimal initializer is too large"))?;
        Self::new(numerator, denominator)
    }

    pub(super) fn to_i128_integer(self) -> CompileResult<i128> {
        if self.denominator != 1 {
            return Err(CompileError::new(
                "non-integer operand in integer initializer",
            ));
        }
        Ok(self.numerator)
    }

    pub(super) fn to_i64_trunc(self) -> CompileResult<i64> {
        i64::try_from(self.numerator / self.denominator)
            .map_err(|_| CompileError::new("integer initializer does not fit i64"))
    }

    pub(super) fn checked_neg(self) -> CompileResult<Self> {
        Self::new(
            self.numerator
                .checked_neg()
                .ok_or_else(|| CompileError::new("integer initializer overflow"))?,
            self.denominator,
        )
    }

    pub(super) fn checked_add(self, right: Self) -> CompileResult<Self> {
        let numerator = self
            .numerator
            .checked_mul(right.denominator)
            .and_then(|left| {
                right
                    .numerator
                    .checked_mul(self.denominator)
                    .and_then(|right| left.checked_add(right))
            })
            .ok_or_else(|| CompileError::new("integer initializer overflow"))?;
        let denominator = self
            .denominator
            .checked_mul(right.denominator)
            .ok_or_else(|| CompileError::new("integer initializer overflow"))?;
        Self::new(numerator, denominator)
    }

    pub(super) fn checked_sub(self, right: Self) -> CompileResult<Self> {
        self.checked_add(right.checked_neg()?)
    }

    pub(super) fn checked_mul(self, right: Self) -> CompileResult<Self> {
        let numerator = self
            .numerator
            .checked_mul(right.numerator)
            .ok_or_else(|| CompileError::new("integer initializer overflow"))?;
        let denominator = self
            .denominator
            .checked_mul(right.denominator)
            .ok_or_else(|| CompileError::new("integer initializer overflow"))?;
        Self::new(numerator, denominator)
    }

    pub(super) fn checked_div(self, right: Self) -> CompileResult<Self> {
        let numerator = self
            .numerator
            .checked_mul(right.denominator)
            .ok_or_else(|| CompileError::new("integer initializer overflow"))?;
        let denominator = self
            .denominator
            .checked_mul(right.numerator)
            .ok_or_else(|| CompileError::new("integer initializer overflow"))?;
        Self::new(numerator, denominator)
    }

    pub(super) fn checked_rem(self, right: Self) -> CompileResult<Self> {
        let left = self.to_i128_integer()?;
        let right = right.to_i128_integer()?;
        if right == 0 {
            return Err(CompileError::new("integer initializer modulo by zero"));
        }
        Self::new(
            left.checked_rem(right)
                .ok_or_else(|| CompileError::new("integer initializer overflow"))?,
            1,
        )
    }

    pub(super) fn checked_shl(self, right: Self) -> CompileResult<Self> {
        let left = self.to_i128_integer()?;
        let right = initializer_shift_count(right)?;
        Self::new(
            left.checked_shl(right)
                .ok_or_else(|| CompileError::new("integer initializer shift overflow"))?,
            1,
        )
    }

    pub(super) fn checked_shr(self, right: Self) -> CompileResult<Self> {
        let left = self.to_i128_integer()?;
        let right = initializer_shift_count(right)?;
        Self::new(
            left.checked_shr(right)
                .ok_or_else(|| CompileError::new("integer initializer shift overflow"))?,
            1,
        )
    }
}

fn initializer_shift_count(value: InitializerNumber) -> CompileResult<u32> {
    let value = value.to_i128_integer()?;
    if value < 0 {
        return Err(CompileError::new(
            "negative integer initializer shift count",
        ));
    }
    u32::try_from(value).map_err(|_| CompileError::new("integer initializer shift count too large"))
}
