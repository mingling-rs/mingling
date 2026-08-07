use colored::Colorize;
use std::collections::VecDeque;

/// Trait for adding markdown formatting to strings
pub trait ColorCode {
    fn parse_color_code(&self) -> String;
}

impl ColorCode for &str {
    fn parse_color_code(&self) -> String {
        parse(self)
    }
}

impl ColorCode for String {
    fn parse_color_code(&self) -> String {
        parse(self)
    }
}

/// Converts a string to colored/formatted text with ANSI escape codes.
///
/// Supported syntax:
/// - Bold: `**text**`
/// - Italic: `*text*`
/// - Underline: `_text_`
/// - Angle-bracketed content: `<text>` (displayed as cyan)
/// - Inline code: `` `text` `` (displayed as green)
/// - Color tags: `[[color_name]]` and `[[/]]` to close color
/// - Escape characters: `\*`, `\<`, `\>`, `` \` ``, `\_` for literal characters
/// - Headings: `# Heading 1`, `## Heading 2`, up to `###### Heading 6`
/// - Blockquote: `> text` (displays a gray background marker at the beginning of the line)
///
/// Color tags support the following color names:
/// Color tags support the following color names:
///
/// | Type                    | Color Names                                                                 |
/// |-------------------------|-----------------------------------------------------------------------------|
/// | Standard colors         | `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`       |
/// | Bright colors           | `bright_black`                                                              |
/// |                         | `bright_red`                                                                |
/// |                         | `bright_green`                                                              |
/// |                         | `bright_yellow`                                                             |
/// |                         | `bright_blue`                                                               |
/// |                         | `bright_magenta`                                                            |
/// |                         | `bright_cyan`                                                               |
/// |                         | `bright_white`                                                              |
/// | Bright color shorthands | `b_black`                                                                   |
/// |                         | `b_red`                                                                     |
/// |                         | `b_green`                                                                   |
/// |                         | `b_yellow`                                                                  |
/// |                         | `b_blue`                                                                    |
/// |                         | `b_magenta`                                                                 |
/// |                         | `b_cyan`                                                                    |
/// |                         | `b_white`                                                                   |
/// | Gray colors             | `gray`/`grey`                                                               |
/// |                         | `bright_gray`/`bright_grey`                                                 |
/// |                         | `b_gray`/`b_grey`                                                           |
///
/// Color tags can be nested, `[[/]]` will close the most recently opened color tag.
///
/// # Arguments
/// * `text` - The text to format, can be any type that implements `AsRef<str>`
///
/// # Returns
/// Returns a `String` containing ANSI escape codes that can display colored/formatted text in ANSI-supported terminals.
///
/// # Examples
/// ```
/// # use mingling_cli::display::markdown;
/// let formatted = markdown("Hello **world**!");
/// println!("{}", formatted);
///
/// let colored = markdown("[[red]]Red text[[/]] and normal text");
/// println!("{}", colored);
///
/// let nested = markdown("[[blue]]Blue [[green]]Green[[/]] Blue[[/]] normal");
/// println!("{}", nested);
/// ```
pub fn parse(text: impl AsRef<str>) -> String {
    let text = text.as_ref();
    let lines: Vec<&str> = text.lines().collect();
    let mut result = String::new();
    let mut content_indent = 0;

    for line in lines {
        // Don't trim the line initially because we need to check if it starts with #
        let trimmed_line = line.trim();
        let mut line_result = String::new();

        // Check if line starts with # for heading
        // Check if the original line (not trimmed) starts with #
        if line.trim_start().starts_with('#') {
            let chars: Vec<char> = line.trim_start().chars().collect();
            let mut level = 0;

            // Count # characters at the beginning
            while level < chars.len() && level < 7 && chars[level] == '#' {
                level += 1;
            }

            // Cap level at 6
            let effective_level = if level > 6 { 6 } else { level };

            // Skip # characters and any whitespace after them
            let mut content_start = level;
            while content_start < chars.len() && chars[content_start].is_whitespace() {
                content_start += 1;
            }

            // Extract heading content
            let heading_content: String = if content_start < chars.len() {
                chars[content_start..].iter().collect()
            } else {
                String::new()
            };

            // Process the heading content with formatting
            let processed_content = process_line(&heading_content);

            // Format heading as white background, black text, bold
            // ANSI codes: \x1b[1m for bold, \x1b[47m for white background, \x1b[30m for black text
            let formatted_heading = format!("\x1b[1m\x1b[47m\x1b[30m {processed_content} \x1b[0m");

            // Add indentation to the heading line itself
            // Heading indentation = level - 1
            let heading_indent = if effective_level > 0 {
                effective_level - 1
            } else {
                0
            };
            let indent = " ".repeat(heading_indent);
            line_result.push_str(&indent);
            line_result.push_str(&formatted_heading);

            // Update content indent level for subsequent content
            // Content after heading should be indented by effective_level
            content_indent = effective_level;
        } else if !trimmed_line.is_empty() {
            // Process regular line with existing formatting
            let processed_line = process_line_with_quote(trimmed_line);

            // Add indentation based on content_indent
            let indent = " ".repeat(content_indent);
            line_result.push_str(&indent);
            line_result.push_str(&processed_line);
        } else {
            line_result.push(' ');
        }

        if !line_result.is_empty() {
            result.push_str(&line_result);
            result.push('\n');
        }
    }

    // Remove trailing newline
    if result.ends_with('\n') {
        result.pop();
    }

    result
}

// Helper function to process a single line with existing formatting and handle > quotes
fn process_line_with_quote(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();

    // Check if line starts with '>' and not escaped
    if !chars.is_empty() && chars[0] == '>' {
        // Check if it's escaped
        if chars.len() > 1 && chars[1] == '\\' {
            // It's \>, so treat as normal text starting from position 0
            return process_line(line);
        }

        // It's a regular > at the beginning, replace with gray background gray text space
        let gray_bg_space = "\x1b[48;5;242m\x1b[38;5;242m \x1b[0m";
        let rest_of_line = if chars.len() > 1 {
            chars[1..].iter().collect::<String>()
        } else {
            String::new()
        };

        // Process the rest of the line normally
        let processed_rest = process_line(&rest_of_line);

        // Combine the gray background space with the processed rest
        format!("{gray_bg_space}{processed_rest}")
    } else {
        // No > at the beginning, process normally
        process_line(line)
    }
}

// Helper function to process a single line with existing formatting
fn process_line(line: &str) -> String {
    let mut result = String::new();
    let mut color_stack: VecDeque<String> = VecDeque::new();

    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check for escape character \
        if chars[i] == '\\' && i + 1 < chars.len() {
            let escaped_char = chars[i + 1];
            // Only escape specific characters
            if matches!(escaped_char, '*' | '<' | '>' | '`' | '_') {
                let mut escaped_text = escaped_char.to_string();
                apply_color_stack(&mut escaped_text, &color_stack);
                result.push_str(&escaped_text);
                i += 2;
                continue;
            }
        }

        // Check for color tag start [[color]]
        if i + 1 < chars.len()
            && chars[i] == '['
            && chars[i + 1] == '['
            && let Some(end) = find_tag_end(&chars, i)
        {
            let tag_content: String = chars[i + 2..end].iter().collect();

            // Check if it's a closing tag [[/]]
            if tag_content == "/" {
                color_stack.pop_back();
            } else {
                // It's a color tag
                color_stack.push_back(tag_content.clone());
            }
            i = end + 2;
            continue;
        }

        // Check for bold **text**
        if i + 1 < chars.len()
            && chars[i] == '*'
            && chars[i + 1] == '*'
            && let Some(end) = find_matching(&chars, i + 2, "**")
        {
            let bold_text: String = chars[i + 2..end].iter().collect();
            let mut formatted_text = bold_text.bold().to_string();
            apply_color_stack(&mut formatted_text, &color_stack);
            result.push_str(&formatted_text);
            i = end + 2;
            continue;
        }

        // Check for italic *text*
        if chars[i] == '*'
            && let Some(end) = find_matching(&chars, i + 1, "*")
        {
            let italic_text: String = chars[i + 1..end].iter().collect();
            let mut formatted_text = italic_text.italic().to_string();
            apply_color_stack(&mut formatted_text, &color_stack);
            result.push_str(&formatted_text);
            i = end + 1;
            continue;
        }

        // Check for underline _text_
        if chars[i] == '_'
            && let Some(end) = find_matching(&chars, i + 1, "_")
        {
            let underline_text: String = chars[i + 1..end].iter().collect();
            let mut formatted_text = format!("\x1b[4m{underline_text}\x1b[0m");
            apply_color_stack(&mut formatted_text, &color_stack);
            result.push_str(&formatted_text);
            i = end + 1;
            continue;
        }

        // Check for angle-bracketed content <text>
        if chars[i] == '<'
            && let Some(end) = find_matching(&chars, i + 1, ">")
        {
            // Include the angle brackets in the output
            let angle_text: String = chars[i..=end].iter().collect();
            let mut formatted_text = angle_text.cyan().to_string();
            apply_color_stack(&mut formatted_text, &color_stack);
            result.push_str(&formatted_text);
            i = end + 1;
            continue;
        }

        // Check for inline code `text`
        if chars[i] == '`'
            && let Some(end) = find_matching(&chars, i + 1, "`")
        {
            // Include the backticks in the output
            let code_text: String = chars[i..=end].iter().collect();
            let mut formatted_text = code_text.green().to_string();
            apply_color_stack(&mut formatted_text, &color_stack);
            result.push_str(&formatted_text);
            i = end + 1;
            continue;
        }

        // Regular character
        let mut current_char = chars[i].to_string();
        apply_color_stack(&mut current_char, &color_stack);
        result.push_str(&current_char);
        i += 1;
    }

    result
}

// Helper function to find matching delimiter
fn find_matching(chars: &[char], start: usize, delimiter: &str) -> Option<usize> {
    let delim_chars: Vec<char> = delimiter.chars().collect();
    let delim_len = delim_chars.len();

    let mut j = start;
    while j < chars.len() {
        if delim_len == 1 {
            if chars[j] == delim_chars[0] {
                return Some(j);
            }
        } else if j + 1 < chars.len()
            && chars[j] == delim_chars[0]
            && chars[j + 1] == delim_chars[1]
        {
            return Some(j);
        }
        j += 1;
    }
    None
}

// Helper function to find color tag end
fn find_tag_end(chars: &[char], start: usize) -> Option<usize> {
    let mut j = start + 2;
    while j + 1 < chars.len() {
        if chars[j] == ']' && chars[j + 1] == ']' {
            return Some(j);
        }
        j += 1;
    }
    None
}

// Helper function to apply color stack to text
fn apply_color_stack(text: &mut String, color_stack: &VecDeque<String>) {
    let mut result = text.clone();
    for color in color_stack.iter().rev() {
        result = apply_color(&result, color);
    }
    *text = result;
}

// Helper function to apply color to text
fn apply_color(text: impl AsRef<str>, color_name: impl AsRef<str>) -> String {
    let text = text.as_ref();
    let color_name = color_name.as_ref();
    match color_name {
        // Normal colors
        "black" => text.black().to_string(),
        "red" => text.red().to_string(),
        "green" => text.green().to_string(),
        "yellow" => text.yellow().to_string(),
        "blue" => text.blue().to_string(),
        "magenta" => text.magenta().to_string(),
        "cyan" => text.cyan().to_string(),
        "white" | "b_white" | "bright_gray" | "bright_grey" | "b_gray" | "b_grey" => {
            text.white().to_string()
        }

        // Bright colors and their b_ short aliases
        "bright_black" | "b_black" | "gray" | "grey" => text.bright_black().to_string(),
        "bright_red" | "b_red" => text.bright_red().to_string(),
        "bright_green" | "b_green" => text.bright_green().to_string(),
        "bright_yellow" | "b_yellow" => text.bright_yellow().to_string(),
        "bright_blue" | "b_blue" => text.bright_blue().to_string(),
        "bright_magenta" | "b_magenta" => text.bright_magenta().to_string(),
        "bright_cyan" | "b_cyan" => text.bright_cyan().to_string(),
        "bright_white" => text.bright_white().to_string(),

        // Default to white if color not recognized
        _ => text.to_string(),
    }
}
