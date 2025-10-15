#[must_use]
pub fn info_text() -> String {
    format!(
        "==========\n{}\n==========",
        include_str!("../../docker/license-info.txt")
    )
}
