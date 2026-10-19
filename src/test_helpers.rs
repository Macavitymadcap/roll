use crate::notation::{Modifier, Operator, Scale, ScaleOperator};

pub(crate) fn modifier(operator: Operator, value: u32) -> Modifier {
    Modifier::try_new(operator, value).unwrap()
}

pub(crate) fn scale(operator: ScaleOperator, value: u32) -> Scale {
    Scale::try_new(operator, value).unwrap()
}

// pub(crate) fn notation(count: u32, sides: u32, modifier: Option<Modifier>) -> Notation {
//     Notation::try_new(count, sides, modifier).unwrap()
// }
