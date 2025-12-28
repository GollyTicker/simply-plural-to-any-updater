use crate::{
    plurality::{
    CleanForPlatform, Fronter, FrontingFormat, VRCHAT_MAX_ALLOWED_STATUS_LENGTH,
    },
    users::{UsePluralKitName, UserConfigForUpdater},
};

use super::{
    clean_name_for_vrchat_status, format_fronting_status, string_unicode_codepoints_length,
};

fn mock_formatter_for_tests(
    prefix: &str,
    no_fronts: &str,
    name_truncate_to: usize,
    max_length: usize,
) -> FrontingFormat {
    FrontingFormat {
        prefix: prefix.to_owned(),
        status_if_no_fronters: no_fronts.to_owned(),
        truncate_names_to_length_if_status_too_long: name_truncate_to,
        cleaning: CleanForPlatform::VRChat,
        max_length: Some(max_length),
    }
}

fn create_test_user_config(use_pluralkit_name: UsePluralKitName) -> UserConfigForUpdater {
    UserConfigForUpdater {
        user_id: crate::users::UserId {
            inner: sqlx::types::Uuid::new_v4(),
        },
        show_members_non_archived: true,
        show_members_archived: false,
        respect_front_notifications_disabled: true,
        privacy_fine_grained: crate::users::PrivacyFineGrained::NoFineGrained,
        privacy_fine_grained_buckets: None,
        client: reqwest::Client::new(),
        simply_plural_base_url: Default::default(),
        discord_base_url: Default::default(),
        status_prefix: Default::default(),
        status_no_fronts: Default::default(),
        status_truncate_names_to: 0,
        show_custom_fronts: false,
        enable_website: false,
        enable_discord: false,
        enable_discord_status_message: false,
        enable_vrchat: false,
        enable_to_pluralkit: false,
        website_url_name: Default::default(),
        website_system_name: Default::default(),
        simply_plural_token: Default::default(),
        discord_status_message_token: Default::default(),
        vrchat_username: Default::default(),
        vrchat_password: Default::default(),
        vrchat_cookie: Default::default(),
        pluralkit_token: Default::default(),
        use_pluralkit_name,
    }
}

// Helper function to create mock Fronter
fn mock_fronter(
    name: &str,
    pk_name: Option<&str>,
    pk_display_name: Option<&str>,
) -> Fronter {
    Fronter {
        fronter_id: String::new(),
        name: name.to_string(),
        avatar_url: String::new(),
        vrchat_status_name: None,
        pluralkit_id: Some("pk_id".to_string()),
        pluralkit_name: pk_name.map(|s| s.to_string()),
        pluralkit_display_name: pk_display_name.map(|s| s.to_string()),
        start_time: None,
        privacy_buckets: vec![],
    }
}

#[test]
fn test_format_vrchat_status_empty_fronts() {
    let config = mock_formatter_for_tests("F:", "nobody?", 3, VRCHAT_MAX_ALLOWED_STATUS_LENGTH);
    let fronts = vec![];
    assert_eq!(format_fronting_status(&config, &fronts), "F: nobody?");
}

#[test]
fn test_format_vrchat_status_single_member_fits_long_string() {
    let config = mock_formatter_for_tests("F:", "N/A", 3, VRCHAT_MAX_ALLOWED_STATUS_LENGTH);
    let fronts = vec![mock_member_content("Alice", "")]; // "P: Alice" (8 chars)
    assert_eq!(format_fronting_status(&config, &fronts), "F: Alice");
}

#[test]
fn test_format_vrchat_status_multiple_members_fit_long_string() {
    let config = mock_formatter_for_tests("F:", "N/A", 3, VRCHAT_MAX_ALLOWED_STATUS_LENGTH);
    let fronts = vec![
        mock_member_content("Alice", ""),
        mock_member_content("Bob", ""),
    ]; // "P: Alice, Bob" (13 chars)
    assert_eq!(format_fronting_status(&config, &fronts), "F: Alice, Bob");
}

#[test]
fn test_format_vrchat_status_fits_short_string_not_long() {
    // VRCHAT_MAX_ALLOWED_STATUS_LENGTH is 23
    let config = mock_formatter_for_tests("Status:", "N/A", 3, VRCHAT_MAX_ALLOWED_STATUS_LENGTH);
    let fronts = vec![
        mock_member_content("UserOne", ""),
        mock_member_content("UserTwo", ""),
    ];
    // Long: "Status: UserOne, UserTwo" (24 chars) > 23
    // Short: "Status:UserOne,UserTwo" (23 chars) <= 23
    assert_eq!(
        format_fronting_status(&config, &fronts),
        "Status:UserOne,UserTwo"
    );
}

#[test]
fn test_format_vrchat_status_fits_truncated_string_not_short() {
    let config = mock_formatter_for_tests("F:", "N/A", 3, VRCHAT_MAX_ALLOWED_STATUS_LENGTH);
    let fronts = vec![
        mock_member_content("Alexander", ""),
        mock_member_content("Benjamin", ""),
        mock_member_content("Charlotte", ""),
    ];
    // Long: "P: Alexander, Benjamin, Charlotte" 33 > 23
    // Short: "P:Alexander,Benjamin,Charlotte" 31 > 23
    // Truncated: "P:Ale,Ben,Cha" 14 <= 23
    assert_eq!(format_fronting_status(&config, &fronts), "F:Ale,Ben,Cha");
}

#[test]
fn test_format_vrchat_status_uses_vrchat_status_name() {
    let config = mock_formatter_for_tests("F:", "N/A", 3, VRCHAT_MAX_ALLOWED_STATUS_LENGTH);
    let fronts = vec![mock_member_content("OriginalName", "VRChatSpecific")];
    assert_eq!(
        format_fronting_status(&config, &fronts),
        "F: VRChatSpecific"
    );
}

#[test]
fn test_format_vrchat_status_cleans_names() {
    let config = mock_formatter_for_tests("F:", "N/A", 3, VRCHAT_MAX_ALLOWED_STATUS_LENGTH);
    let fronts = vec![mock_member_content("User😊Name", "")];
    assert_eq!(format_fronting_status(&config, &fronts), "F: UserName");
}

#[test]
fn test_format_vrchat_status_doesnt_clean_specifically_configured_name() {
    let config = mock_formatter_for_tests("F:", "N/A", 3, VRCHAT_MAX_ALLOWED_STATUS_LENGTH);
    let fronts = vec![mock_member_content("UN", "User😊Name")];
    assert_eq!(format_fronting_status(&config, &fronts), "F: User😊Name");
}

#[test]
fn test_format_vrchat_status_complex_truncation_and_vrc_name() {
    let config = mock_formatter_for_tests("F:", "N/A", 4, VRCHAT_MAX_ALLOWED_STATUS_LENGTH);
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
    assert_eq!(format_fronting_status(&config, &fronts), "F:Long,VRC1,Anot");
}

#[test]
fn test_format_status_truncation() {
    let config = mock_formatter_for_tests("F:", "N/A", 4, 10);
    let fronts = vec![
        mock_member_content("LongNameOne😊", ""),
        mock_member_content("Shorty", "VRC11"),
        mock_member_content("AnotherVeryLong", ""),
    ];
    // Cleaned names for status: LongNameOne, VRC11, AnotherVeryLong
    // Truncated names: Long, VRC1, Anot
    // Truncated string: "F:Long,VRC1,Anot" 17 > 10
    // Count: "F: 3#" 5 <= 10
    assert_eq!(format_fronting_status(&config, &fronts), "F: 3#");
}

#[test]
fn length_counts_codepoints_and_not_bytes() {
    assert_eq!(string_unicode_codepoints_length("123"), 3);
    assert_eq!(string_unicode_codepoints_length("é"), 1);
    assert_eq!(string_unicode_codepoints_length("你好"), 2);
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
