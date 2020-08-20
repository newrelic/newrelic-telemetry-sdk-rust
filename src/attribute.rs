/// Represents any valid attribute value.
///
/// According to the [specification](https://github.com/newrelic/newrelic-telemetry-sdk-specs/blob/master/capabilities.md),
/// attribute values can be a string, numeric, or boolean. A numeric value is
/// represented either as a signed integer, an unsigned integer or a float.
#[derive(serde::Serialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Value {
    /// Represents a signed integer attribute value.
    ///
    /// ```
    /// # use newrelic_telemetry::attribute::Value;
    /// #
    /// let v = Value::Int(-3);
    /// ```
    Int(i64),

    /// Represents an unsigned integer attribute value.
    ///
    /// ```
    /// # use newrelic_telemetry::attribute::Value;
    /// #
    /// let v = Value::UInt(42);
    /// ```
    UInt(u64),

    /// Represents a string attribute value.
    ///
    /// ```
    /// # use newrelic_telemetry::attribute::Value;
    /// #
    /// let v = Value::Str(String::from("root"));
    /// ```
    Str(String),

    /// Represents a float attribute value.
    ///
    /// ```
    /// # use newrelic_telemetry::attribute::Value;
    /// #
    /// let v = Value::Float(3.14159);
    /// ```
    Float(f64),

    /// Represents a bool attribute value.
    ///
    /// ```
    /// # use newrelic_telemetry::attribute::Value;
    /// #
    /// let v = Value::Bool(true);
    /// ```
    Bool(bool),
}

/// Types whose values can be converted to attribute values.
///
/// If a type implements `ToValue`, it can be converted into an attribute value:
/// ```
/// # use newrelic_telemetry::attribute::{Value, ToValue};
/// #
/// let duration = 3.14159;
/// let name = "root";
///
/// let mut v: Value = duration.to_attribute_value();
/// v = name.to_attribute_value();
/// ```
pub trait ToValue {
    fn to_attribute_value(&self) -> Value;
}

/// Converts an i64 to an attribute value.
///
/// ```
/// # use newrelic_telemetry::attribute::{Value, ToValue};
/// #
/// let v: i64 = -5;
/// assert_eq!(v.to_attribute_value(), Value::Int(-5));
/// ```
impl ToValue for i64 {
    fn to_attribute_value(&self) -> Value {
        Value::Int(*self)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Value {
        Value::Int(value)
    }
}

/// Converts an i32 to an attribute value.
///
/// ```
/// # use newrelic_telemetry::attribute::{Value, ToValue};
/// #
/// let v: i32 = -5;
/// assert_eq!(v.to_attribute_value(), Value::Int(-5));
/// ```
impl ToValue for i32 {
    fn to_attribute_value(&self) -> Value {
        Value::Int(*self as i64)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Value {
        Value::Int(value as i64)
    }
}

/// Converts a u64 to an attribute value.
///
/// ```
/// # use newrelic_telemetry::attribute::{Value, ToValue};
/// #
/// let v: u64 = 5;
/// assert_eq!(v.to_attribute_value(), Value::UInt(5));
/// ```
impl ToValue for u64 {
    fn to_attribute_value(&self) -> Value {
        Value::UInt(*self)
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Value {
        Value::UInt(value)
    }
}

/// Converts a u32 to an attribute value.
///
/// ```
/// # use newrelic_telemetry::attribute::{Value, ToValue};
/// #
/// let v: u32 = 5;
/// assert_eq!(v.to_attribute_value(), Value::UInt(5));
/// ```
impl ToValue for u32 {
    fn to_attribute_value(&self) -> Value {
        Value::UInt(*self as u64)
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Value {
        Value::UInt(value as u64)
    }
}

/// Converts a string to an attribute value.
///
/// ```
/// # use newrelic_telemetry::attribute::{Value, ToValue};
/// #
/// let v = "root";
/// assert_eq!(v.to_attribute_value(), Value::Str(String::from("root")));
/// ```
impl ToValue for &str {
    fn to_attribute_value(&self) -> Value {
        Value::Str(self.to_string())
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Value {
        Value::Str(value.to_string())
    }
}

/// Converts a f64 to an attribute value.
///
/// ```
/// # use newrelic_telemetry::attribute::{Value, ToValue};
/// #
/// let v: f64 = 3.14159;
/// assert_eq!(v.to_attribute_value(), Value::Float(v));
/// ```
impl ToValue for f64 {
    fn to_attribute_value(&self) -> Value {
        Value::Float(*self)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Value {
        Value::Float(value)
    }
}

/// Converts a f32 to an attribute value.
///
/// ```
/// # use newrelic_telemetry::attribute::{Value, ToValue};
/// #
/// let v: f32 = 3.14159;
/// assert_eq!(v.to_attribute_value(), Value::Float(v as f64));
/// ```
impl ToValue for f32 {
    fn to_attribute_value(&self) -> Value {
        Value::Float(*self as f64)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Value {
        Value::Float(value as f64)
    }
}

/// Converts a bool to an attribute value.
///
/// ```
/// # use newrelic_telemetry::attribute::{Value, ToValue};
/// #
/// let v = true;
/// assert_eq!(v.to_attribute_value(), Value::Bool(true));
/// ```
impl ToValue for bool {
    fn to_attribute_value(&self) -> Value {
        Value::Bool(*self)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Value {
        Value::Bool(value)
    }
}

#[cfg(test)]
mod tests {
    use super::Value;
    use serde_json::json;

    #[test]
    fn value_to_json() {
        // Attribute values should serialize to plain JSON values.
        assert_eq!(json!(Value::Int(-5)), json!(-5));
        assert_eq!(json!(Value::UInt(5)), json!(5));
        assert_eq!(json!(Value::Float(3.14159)), json!(3.14159));
        assert_eq!(json!(Value::Str(String::from("root"))), json!("root"));
        assert_eq!(json!(Value::Bool(true)), json!(true));
    }

    #[test]
    fn into_value() {
        // Should be able to use Value::from or .into() to create Values
        assert_eq!(Value::Int(-5), Value::from(-5));
        assert_eq!(Value::Int(-5), (-5 as i32).into());

        // cast needed because integer types default to i32
        assert_eq!(Value::UInt(5), Value::from(5 as u64));
        assert_eq!(Value::UInt(5), (5 as u64).into());

        assert_eq!(Value::Float(3.14159), Value::from(3.14159));
        assert_eq!(Value::Float(3.14159), (3.14159 as f64).into());

        assert_eq!(Value::Str("root".to_string()), Value::from("root"));
        assert_eq!(Value::Str("root".to_string()), "root".into());

        assert_eq!(Value::Bool(true), Value::from(true));
        assert_eq!(Value::Bool(true), true.into());
    }
}
