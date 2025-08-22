use crate::plurality::Fronter;

use encoding_rs::ISO_8859_15;

pub const VRCHAT_MAX_ALLOWED_STATUS_LENGTH: usize = 23;
pub const DISCORD_STATUS_MAX_LENGTH: usize = 128;

pub struct FrontingFormat {
    pub max_length: Option<usize>,
    pub cleaning: CleanForPlatform,
    pub prefix: String,
    pub status_if_no_fronters: String,
    pub truncate_names_to_length_if_status_too_long: usize,
}

pub enum CleanForPlatform {
    NoClean,
    VRChat,
}

pub fn format_fronting_status(fronting_format: &FrontingFormat, fronts: &[Fronter]) -> String {
    let cleaned_fronter_names = collect_clean_fronter_names(fronting_format, fronts);
    eprintln!("Cleaned fronter names for status: {cleaned_fronter_names:?}");

    let status_strings =
        compute_status_strings_of_decreasing_lengths_for_aesthetics_and_information_tradeoff(
            fronting_format,
            &cleaned_fronter_names,
        );

    pick_longest_string_within_vrchat_status_length_limit(fronting_format, &status_strings)
}

fn collect_clean_fronter_names(
    fronting_format: &FrontingFormat,
    fronts: &[Fronter],
) -> Vec<String> {
    if fronts.is_empty() {
        vec![fronting_format.status_if_no_fronters.clone()] // Use configured string if no fronters
    } else {
        fronts
            .iter()
            .map(|f| match fronting_format.cleaning {
                CleanForPlatform::NoClean => f.preferred_vrchat_status_name(),
                CleanForPlatform::VRChat => {
                    clean_name_for_vrchat_status(&f.preferred_vrchat_status_name())
                }
            })
            .collect()
    }
}

fn compute_status_strings_of_decreasing_lengths_for_aesthetics_and_information_tradeoff(
    fronting_format: &FrontingFormat,
    fronter_names: &[String],
) -> Vec<String> {
    // Convert Vec<String> to Vec<&str> for convenient joining and slicing.
    let fronter_names_as_str: Vec<&str> = fronter_names.iter().map(String::as_str).collect();

    let long_string = format!(
        "{} {}",
        fronting_format.prefix,
        fronter_names_as_str.join(", ")
    );
    let short_string = format!(
        "{}{}",
        fronting_format.prefix,
        fronter_names_as_str.join(",")
    );
    let truncated_string = {
        let truncated_names: Vec<String> = fronter_names_as_str
            .iter()
            .map(|&name| {
                let mut truncated_name = String::new();

                let () = &name
                    .chars()
                    .take(fronting_format.truncate_names_to_length_if_status_too_long)
                    .for_each(|c| truncated_name.push(c));

                truncated_name
            })
            .collect();
        format!(
            "{}{}",
            fronting_format.prefix,
            truncated_names.join(",").as_str()
        )
    };
    let count_string = format!("{} {}#", fronting_format.prefix, fronter_names.len());

    eprintln!(
        "Long      string: '{}' ({})",
        long_string,
        long_string.len()
    );
    eprintln!(
        "Short     string: '{}' ({})",
        short_string,
        short_string.len()
    );
    eprintln!(
        "Truncated string: '{}' ({})",
        truncated_string,
        truncated_string.len()
    );

    eprintln!(
        "Count     string: '{}' ({})",
        count_string,
        count_string.len()
    );

    vec![long_string, short_string, truncated_string, count_string]
}

fn pick_longest_string_within_vrchat_status_length_limit(
    fronting_format: &FrontingFormat,
    status_strings: &[String],
) -> String {
    let empty_string = String::new();
    status_strings
        .iter()
        .filter(|s| fronting_format.max_length.is_none_or(|l| s.len() <= l))
        .max_by_key(|s| s.len())
        .unwrap_or(&empty_string) // can't happen due to compile time guarantee
        .to_string()
}

// VRChat status messages does not display all UTF-8 characters.
// This function removes all characters which are not of a specific encoding from the string.
// We also trim the name, in case the cleanup made new spaces appear.
pub fn clean_name_for_vrchat_status(dirty_name: &str) -> String {
    let mut iso_filtered_name = String::new();

    for ch in dirty_name.chars() {
        // Convert char utf-8 str
        let ch_string = ch.to_string();

        // convert utf-8 str to the limited encoding and check if the character is supported.
        let mut char_cleaned_buffer = [0u8; 20];
        let (_, _, _, is_unsupported_character) = ISO_8859_15.new_encoder().encode_from_utf8(
            ch_string.as_str(),
            &mut char_cleaned_buffer,
            true,
        );

        if !is_unsupported_character {
            iso_filtered_name.push(ch);
        }
    }

    // remove consecutive whitespace resulting from cleanup. also trims string.
    iso_filtered_name
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}
