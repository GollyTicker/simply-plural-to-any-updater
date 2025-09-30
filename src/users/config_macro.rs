#[macro_export]
macro_rules! config_value {
    ($config:expr, $field:ident) => {
        $config.$field.clone().ok_or_else(|| {
            anyhow!(format!(
                "config_value: Mandatory field undefined or invalid: '{}'",
                stringify!($field)
            ))
        })
    };
}

#[macro_export]
macro_rules! config_value_if {
    ($condition:expr, $config:expr, $field:ident) => {
        if $condition {
            config_value!($config, $field)
        } else {
            Ok(Default::default())
        }
    };
}
