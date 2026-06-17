mod group0;
mod group1;
mod group2;
mod group3;
pub(super) fn rules() -> Vec<crate::Rule> {
    let mut output = Vec::new();
    output.extend(group0::rules());
    output.extend(group1::rules());
    output.extend(group2::rules());
    output.extend(group3::rules());

    output
}
