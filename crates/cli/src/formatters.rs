//! Shared text formatting utilities for consistent styling

use console::style;
use std::fmt::Display;

/// Format text with key field styling (bold and cyan when colors enabled)
pub(crate) fn format_key<T: Display>(text: T, use_colors: bool) -> String {
    let text = text.to_string();
    let styled = style(text).bold();

    if use_colors {
        styled.cyan().to_string()
    } else {
        styled.to_string()
    }
}
