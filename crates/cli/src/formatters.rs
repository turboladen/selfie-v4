//! Shared text formatting utilities for consistent styling

use console::style;
use std::fmt::Display;

/// Styling options for different field types
pub(crate) enum FieldStyle {
    Key,   // Table keys, field names
    Value, // Values in tables/output
    Title, // Section titles
}

/// Format text with consistent styling based on field type
pub(crate) fn format_field<T: Display>(
    text: T,
    style_type: FieldStyle,
    use_colors: bool,
) -> String {
    let text = text.to_string();

    // Apply bold styling regardless of colors for keys and titles
    let styled = match style_type {
        FieldStyle::Key | FieldStyle::Title => style(text).bold(),
        _ => style(text),
    };

    if !use_colors {
        return styled.to_string();
    }

    // Apply color styling
    match style_type {
        FieldStyle::Key => styled.cyan().to_string(),
        FieldStyle::Value => styled.white().to_string(),
        FieldStyle::Title => styled.magenta().to_string(),
    }
}
