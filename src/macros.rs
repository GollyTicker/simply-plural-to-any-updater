// For config values
#[macro_export]
macro_rules! config_value {
    ($config:expr, $field:ident) => {
        $config.$field.clone().ok_or_else(|| {
            anyhow!(format!(
                "Mandatory field undefined or invalid: '{}'",
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

// For Future Runtime
#[macro_export]
macro_rules! record_if_error {
    ($self:expr, $result:expr) => {{
        let operation = $result;
        $self.last_operation_error = operation.as_ref().err().map(|e| e.to_string());
        operation
    }};
}
