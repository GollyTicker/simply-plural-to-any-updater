use crate::{config::Config, simply_plural};

use encoding_rs::ISO_8859_15;

const VRCHAT_MAX_ALLOWED_STATUS_LENGTH: usize = 23;

pub(crate) fn format_fronts_for_vrchat_status(
    config: &Config,
    fronts: Vec<simply_plural::Fronter>,
) -> String {
    let cleaned_fronter_names = clean_fronter_names(fronts, &config.vrchat_updater_no_fronts);
    eprintln!(
        "Cleaned fronter names for status: {:?}",
        cleaned_fronter_names
    );

    let status_strings =
        compute_status_strings_of_decreasing_lengths_for_aesthetics_and_information_tradeoff(
            config,
            cleaned_fronter_names,
        );

    pick_longest_string_within_vrchat_status_length_limit(status_strings)
}

fn clean_fronter_names(
    fronts: Vec<simply_plural::Fronter>,
    name_if_no_fronters: &String,
) -> Vec<String> {
    if fronts.is_empty() {
        vec![name_if_no_fronters.clone()] // Use configured string if no fronters
    } else {
        fronts
            .iter()
            .map(|f| f.vrchat_status_name.clone().unwrap_or_else(||clean_name_for_vrchat_status(&f.name)))
            .collect()
    }
}

fn compute_status_strings_of_decreasing_lengths_for_aesthetics_and_information_tradeoff(
    config: &Config,
    fronter_names: Vec<String>,
) -> Vec<String> {
    // Convert Vec<String> to Vec<&str> for convenient joining and slicing.
    let fronter_names_as_str: Vec<&str> = fronter_names.iter().map(String::as_str).collect();

    let long_string = format!(
        "{} {}",
        config.vrchat_updater_prefix.as_str(),
        fronter_names_as_str.join(", ")
    );
    let short_string = format!(
        "{}{}",
        config.vrchat_updater_prefix.as_str(),
        fronter_names_as_str.join(",")
    );
    let truncated_string = {
        let truncated_names: Vec<String> = fronter_names_as_str
            .iter()
            .map(|&name| {
                let mut truncated_name = String::new();

                let _ = &name
                    .chars()
                    .take(config.vrchat_updater_truncate_names_to)
                    .for_each(|c| truncated_name.push(c));

                truncated_name
            })
            .collect();
        format!(
            "{}{}",
            config.vrchat_updater_prefix.as_str(),
            truncated_names.join(",").as_str()
        )
    };

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

    vec![long_string, short_string, truncated_string]
}

fn pick_longest_string_within_vrchat_status_length_limit(status_strings: Vec<String>) -> String {
    status_strings
        .iter()
        .filter(|s| s.len() <= VRCHAT_MAX_ALLOWED_STATUS_LENGTH)
        .max_by_key(|s| s.len())
        .unwrap()
        .to_string()
}

// VRChat status messages does not display all UTF-8 characters.
// This function removes all characters which are not of a specific encoding from the string.
// We also trim the name, in case the cleanup made new spaces appear.
fn clean_name_for_vrchat_status(dirty_name: &str) -> String {
    let mut iso_filtered_name = String::new();

    for ch in dirty_name.chars() {
        // Convert char utf-8 str
        let ch_string = ch.to_string();

        // convert utf-8 str to the limited encoding and check if the character is supported.
        let mut char_cleaned_buffer = [0u8; 20];
        let (_, _, _, is_unsupported_character) = ISO_8859_15.new_encoder().encode_from_utf8(
            &ch_string.as_str(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::simply_plural::Fronter;

    fn mock_config_for_format_tests(
        prefix: &str,
        no_fronts: &str,
        name_truncate_to: usize,
    ) -> Config {
        let mut config = Config::default();
        config.vrchat_updater_prefix = prefix.to_string();
        config.vrchat_updater_no_fronts = no_fronts.to_string();
        config.vrchat_updater_truncate_names_to = name_truncate_to;
        config
    }

    // Helper function to create mock MemberContent
    fn mock_member_content(name: &str, vrchat_status_name: &str) -> Fronter {
        Fronter {
            id: String::new(),
            name: name.to_string(),
            avatar_url: String::new(),
            vrchat_status_name: if vrchat_status_name.is_empty() {
                None
            } else {
                Some(vrchat_status_name.to_owned())
            },
        }
    }

    #[test]
    fn test_format_vrchat_status_empty_fronts() {
        let config = mock_config_for_format_tests("F:", "nobody?", 3);
        let fronts = vec![];
        assert_eq!(
            format_fronts_for_vrchat_status(&config, fronts),
            "F: nobody?"
        );
    }

    #[test]
    fn test_format_vrchat_status_single_member_fits_long_string() {
        let config = mock_config_for_format_tests("F:", "N/A", 3);
        let fronts = vec![mock_member_content("Alice", "")]; // "P: Alice" (8 chars)
        assert_eq!(format_fronts_for_vrchat_status(&config, fronts), "F: Alice");
    }

    #[test]
    fn test_format_vrchat_status_multiple_members_fit_long_string() {
        let config = mock_config_for_format_tests("F:", "N/A", 3);
        let fronts = vec![
            mock_member_content("Alice", ""),
            mock_member_content("Bob", ""),
        ]; // "P: Alice, Bob" (13 chars)
        assert_eq!(
            format_fronts_for_vrchat_status(&config, fronts),
            "F: Alice, Bob"
        );
    }

    #[test]
    fn test_format_vrchat_status_fits_short_string_not_long() {
        // VRCHAT_MAX_ALLOWED_STATUS_LENGTH is 23
        let config = mock_config_for_format_tests("Status:", "N/A", 3);
        let fronts = vec![
            mock_member_content("UserOne", ""),
            mock_member_content("UserTwo", ""),
        ];
        // Long: "Status: UserOne, UserTwo" (24 chars) > 23
        // Short: "Status:UserOne,UserTwo" (23 chars) <= 23
        assert_eq!(
            format_fronts_for_vrchat_status(&config, fronts),
            "Status:UserOne,UserTwo"
        );
    }

    #[test]
    fn test_format_vrchat_status_fits_truncated_string_not_short() {
        let config = mock_config_for_format_tests("F:", "N/A", 3);
        let fronts = vec![
            mock_member_content("Alexander", ""),
            mock_member_content("Benjamin", ""),
            mock_member_content("Charlotte", ""),
        ];
        // Long: "P: Alexander, Benjamin, Charlotte" 33 > 23
        // Short: "P:Alexander,Benjamin,Charlotte" 31 > 23
        // Truncated: "P:Ale,Ben,Cha" 14 <= 23
        assert_eq!(
            format_fronts_for_vrchat_status(&config, fronts),
            "F:Ale,Ben,Cha"
        );
    }

    #[test]
    fn test_format_vrchat_status_uses_vrchat_status_name() {
        let config = mock_config_for_format_tests("F:", "N/A", 3);
        let fronts = vec![mock_member_content("OriginalName", "VRChatSpecific")];
        assert_eq!(
            format_fronts_for_vrchat_status(&config, fronts),
            "F: VRChatSpecific"
        );
    }

    #[test]
    fn test_format_vrchat_status_cleans_names() {
        let config = mock_config_for_format_tests("F:", "N/A", 3);
        let fronts = vec![mock_member_content("User😊Name", "")];
        assert_eq!(
            format_fronts_for_vrchat_status(&config, fronts),
            "F: UserName"
        );
    }

    #[test]
    fn test_format_vrchat_status_doesnt_clean_specifically_configured_name() {
        let config = mock_config_for_format_tests("F:", "N/A", 3);
        let fronts = vec![mock_member_content("ABC", "User😊Name")];
        assert_eq!(
            format_fronts_for_vrchat_status(&config, fronts),
            "F: User😊Name"
        );
    }

    #[test]
    fn test_format_vrchat_status_complex_truncation_and_vrc_name() {
        let config = mock_config_for_format_tests("F:", "N/A", 4);
        let fronts = vec![
            mock_member_content("LongNameOne😊", ""),
            mock_member_content("Shorty", "VRC11"),
            mock_member_content("AnotherVeryLong", ""),
        ];
        // Cleaned names for status: LongNameOne, VRC11, AnotherVeryLong
        // Long: "F: LongNameOne, VRC11, AnotherVeryLong" 38 > 23
        // Short: "F:LongNameOne,VRC11,AnotherVeryLong" 36 > 23
        // Truncated names: Long, VRC1, Anot
        // Truncated string: "F:Long,VRC1,Anot" 17 <= 23
        assert_eq!(
            format_fronts_for_vrchat_status(&config, fronts),
            "F:Long,VRC1,Anot"
        );
    }

    #[test]
    fn test_clean_name_for_vrchat_encoding_and_whitespace() {
        assert_eq!(
            clean_name_for_vrchat_status("ValidName123!€ Špecial Chars Ž"),
            "ValidName123!€ Špecial Chars Ž",
            "Should keep all valid ISO_8859_15 characters"
        );

        assert_eq!(
            clean_name_for_vrchat_status("Name😊With🚀Emojis❤️Symbols✅"),
            "NameWithEmojisSymbols",
            "Should remove emojis"
        );

        assert_eq!(
            clean_name_for_vrchat_status("Héllo Wörld🎉"),
            "Héllo Wörld",
            "Should handle mixed valid and invalid characters"
        );

        assert_eq!(
            clean_name_for_vrchat_status("  Trimmed  From  Name  "),
            "Trimmed From Name",
            "Should collapse consecutive spaces and trim"
        );

        assert_eq!(clean_name_for_vrchat_status(""), "");

        assert_eq!(clean_name_for_vrchat_status("😊🚀🎉"), "");

        assert_eq!(clean_name_for_vrchat_status("   \t\n   "), "");

        assert_eq!(
            clean_name_for_vrchat_status("你好WorldПриветUser1"),
            "WorldUser1",
            "Should remove characters from other scripts like Hanzi or Cyrillic"
        );

        assert_eq!(
            clean_name_for_vrchat_status("A 😊B C🚀D"),
            "A B CD",
            "Should collapse spaces created by invalid characters"
        );
    }
}
