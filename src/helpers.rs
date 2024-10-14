use crate::*;

impl TryFrom<i32> for WarningDomain {
    type Error = VirtLintError;

    fn try_from(value: i32) -> Result<Self, VirtLintError> {
        let ret = match value {
            0 => WarningDomain::Domain,
            1 => WarningDomain::Node,
            _ => {
                return Err(VirtLintError::InvalidArgument("Unknown warning domain"));
            }
        };

        Ok(ret)
    }
}

impl TryFrom<i32> for WarningLevel {
    type Error = VirtLintError;

    fn try_from(value: i32) -> Result<Self, VirtLintError> {
        let ret = match value {
            0 => WarningLevel::Error,
            1 => WarningLevel::Warning,
            2 => WarningLevel::Notice,
            _ => {
                return Err(VirtLintError::InvalidArgument("Unknown warning level"));
            }
        };

        Ok(ret)
    }
}
