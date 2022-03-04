/// These errors are for internal IR failures, not designed to be useful to a Sway developer, but
/// more for users of the `sway-ir` crate, i.e., compiler developers.
///
/// XXX They're not very rich and could do with a little more verbosity.

#[derive(Debug)]
pub enum IrError {
    FunctionLocalClobbered(String, String),
    InvalidMetadatum,
    MismatchedReturnTypes(String),
    MisplacedTerminator(String),
    MissingBlock(String),
    MissingTerminator(String),
    NonUniquePhiLabels,
    ParseFailure(String, String),
    ValueNotFound(String),

    VerifyBranchToMissingBlock(String),
    VerifyCallToMissingFunction(String),
    VerifyArgumentValueIsNotArgument(String),
    VerifyUntypedValuePassedToFunction,
    VerifyCallArgTypeMismatch(String),
}

use std::fmt;

impl fmt::Display for IrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            IrError::FunctionLocalClobbered(fn_str, var_str) => write!(
                f,
                "Local storage for function {fn_str} already has an entry for variable {var_str}"
            ),
            IrError::InvalidMetadatum => write!(f, "Unable to convert from invalid metadatum."),
            IrError::MismatchedReturnTypes(fn_str) => write!(
                f,
                "Function {fn_str} return type must match its RET instructions."
            ),
            IrError::MisplacedTerminator(blk_str) => {
                write!(f, "Block {blk_str} has a misplaced terminator.")
            }
            IrError::MissingBlock(blk_str) => write!(f, "Unable to find block {blk_str}."),
            IrError::MissingTerminator(blk_str) => {
                write!(f, "Block {blk_str} is missing its terminator.")
            }
            IrError::NonUniquePhiLabels => write!(f, "PHI must have unique block labels."),
            IrError::ParseFailure(expecting, found) => {
                write!(f, "Parse failure: expecting '{expecting}', found '{found}'")
            }
            IrError::ValueNotFound(reason) => {
                write!(f, "Invalid value: {reason}")
            }

            IrError::VerifyBranchToMissingBlock(label) => {
                write!(
                    f,
                    "Branch to block '{label}' is not a block in the current function."
                )
            }
            IrError::VerifyCallToMissingFunction(callee) => {
                write!(f, "Call to invalid function '{callee}'.")
            }
            IrError::VerifyArgumentValueIsNotArgument(callee) => write!(
                f,
                "Argument specifier for function '{callee}' is not an argument value."
            ),
            IrError::VerifyUntypedValuePassedToFunction => write!(
                f,
                "An untyped/void value has been passed to a function call."
            ),
            IrError::VerifyCallArgTypeMismatch(callee) => {
                write!(f, "Type mismatch found for call to '{callee}'.")
            }
        }
    }
}
