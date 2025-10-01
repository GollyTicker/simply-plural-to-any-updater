#[must_use]
pub fn info_text() -> String {
    format!(
        "==========\n{}\n==========",
        include_str!("../../docker/license-info.txt")
    )
}
#[must_use]
pub fn info_short_html() -> String {
    include_str!("../../docker/license-info-short.html").to_owned()
}
