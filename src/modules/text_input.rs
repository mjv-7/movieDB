/*
Made by: Mathew Dusome
April 1 2026 Second Release
Adds a text input object

In your mod.rs file located in the modules folder add the following to the end of the file
        pub mod text_input;


Add with the other use statements
    use crate::modules::text_input::TextInput;

Then to use this you would put the following above the loop: 
    let mut txt_input = TextInput::new(100.0, 100.0, 300.0, 40.0, 25.0);
Where the parameters are x, y, width, height, font size


You can customize the text box using various methods:

LIMITS AND MULTILINE:
    // Set a maximum number of characters
    txt_input.set_max_chars(50);

    // Restrict input to specific characters
    txt_input.set_allowed_chars("0123456789");

    // Enable multiline mode (text wraps within the box)
    txt_input.set_multiline(true);


APPEARANCE CUSTOMIZATION:
    // Set colors (text, border, background, cursor)
    txt_input.with_colors(WHITE, BLUE, DARKGRAY, RED);
    
    // Set individual colors
    txt_input.set_text_color(WHITE)
          .set_border_color(BLUE)
          .set_background_color(DARKGRAY)
          .set_cursor_color(RED);
    
    // Set custom font
    txt_input.with_font(my_font.clone());
    
    // Change position and dimensions
    txt_input.set_position(150.0, 150.0);
    txt_input.set_dimensions(250.0, 50.0);
    
    // Set prompt text and color (shown when input is empty)
    txt_input.set_prompt("Enter your name...");
    txt_input.set_prompt_color(DARKGRAY);

    // Enable or disable the text input
    txt_input.set_enabled(false); // Disable the text input (becomes read-only)
    txt_input.set_enabled(true);  // Enable the text input
    txt_input.set_disabled_color(Color::new(0.7, 0.7, 0.7, 0.5)); // Customize disabled appearance
    
TEXT MANIPULATION:
    // Get current text
    let current_text = txt_input.get_text();
    
    // Set text content
    txt_input.set_text("Hello World");
    
    // Check active state
    if txt_input.is_active() {
        // Do something when textbox is active
    }
    
    // Set cursor position
    txt_input.set_cursor_index(5);

    // Customize key repeat behavior (for arrow keys, backspace, delete)
    txt_input.set_key_repeat_delay(0.3);    // Initial delay before key repeat starts (seconds)
    txt_input.set_key_repeat_rate(0.03);    // Time between repeats after initial delay (seconds)
    // Or set both at once
    txt_input.with_key_repeat_settings(0.3, 0.03);

Then in the main loop you would use:
    // Update and draw the textbox in one step
    txt_input.draw();
*/
use macroquad::prelude::*;
#[cfg(feature = "scale")]
use crate::modules::scale::mouse_position_world as mouse_position;

pub struct TextInput {
        // For vertical navigation, store the preferred column
        preferred_col: Option<usize>,
    // Make all fields private for complete encapsulation
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    text: String,
    active: bool,
    cursor_index: usize,
    cursor_timer: f32,
    cursor_visible: bool,
    font_size: f32,
    text_color: Color,
    border_color: Color,
    background_color: Color,
    cursor_color: Color,
    font: Option<Font>,
    prompt: Option<String>, // New field for prompt text
    prompt_color: Color,    // Color for the prompt text
    // Add key repeat functionality
    key_repeat_delay: f32,  // Initial delay before key starts repeating (in seconds)
    key_repeat_rate: f32,   // How often the key repeats after initial delay (in seconds) 
    key_repeat_timer: f32,  // Timer to track key repeat
    last_key: Option<KeyCode>, // Track the last key that was pressed
    enabled: bool,          // Controls whether the text input can be interacted with
    disabled_color: Color,  // Color used when the text input is disabled
    // New: Multiline and max chars support
    multiline: bool,        // If true, wraps text to next line within box
    max_chars: Option<usize>, // Optional maximum number of characters
    allowed_chars: Option<String>, // Optional whitelist of allowed typed characters
}

impl TextInput {
    fn is_char_allowed(&self, c: char) -> bool {
        self.allowed_chars
            .as_ref()
            .map_or(true, |allowed| allowed.contains(c))
    }

    fn apply_text_constraints(&self, text: &str) -> String {
        let mut constrained = String::new();

        for c in text.chars() {
            if c == '\n' && self.multiline {
                constrained.push(c);
            } else if self.is_char_allowed(c) {
                constrained.push(c);
            }

            if self
                .max_chars
                .is_some_and(|max| constrained.chars().count() >= max)
            {
                break;
            }
        }

        constrained
    }

    fn can_insert_char(&self, c: char) -> bool {
        if c != '\n' && !self.is_char_allowed(c) {
            return false;
        }

        self.max_chars
            .map_or(true, |max| self.text.chars().count() < max)
    }

    /// Returns (wrapped_lines, mapping) where mapping[byte_idx] = (line, col)
    fn get_wrapped_lines_and_mapping(&self) -> (Vec<String>, Vec<(usize, usize)>) {
        if !self.multiline {
            // Single line: mapping is just (0, col) for each char
            let mut mapping = Vec::new();
            let mut col = 0;
            for (i, c) in self.text.char_indices() {
                mapping.resize(i + c.len_utf8(), (0, col));
                col += 1;
            }
            mapping.resize(self.text.len() + 1, (0, col));
            return (vec![self.text.clone()], mapping);
        }
        let mut lines = Vec::new();
        let mut mapping = vec![(0, 0); self.text.len() + 1];
        let padding = 5.0;
        let max_width = self.width - 2.0 * padding;
        let font = self.font.as_ref();
        let mut line_idx: usize = 0;
        let mut col_idx = 0;
        let mut current_line = String::new();
        let mut current_width = 0.0;
        let mut byte_idx = 0;
        let mut chars = self.text.chars().peekable();
        while let Some(c) = chars.next() {
            let c_width = measure_text(&c.to_string(), font, self.font_size as u16, 1.0).width;
            let is_newline = c == '\n';
            // If wrapping needed (but not for newline)
            if !is_newline && current_width + c_width > max_width && !current_line.is_empty() {
                lines.push(current_line.clone());
                current_line.clear();
                current_width = 0.0;
                line_idx += 1;
                col_idx = 0;
            }
            if is_newline {
                lines.push(current_line.clone());
                current_line.clear();
                current_width = 0.0;
                line_idx += 1;
                col_idx = 0;
                mapping[byte_idx] = (line_idx, col_idx); // Map the newline byte
                byte_idx += c.len_utf8();
                continue;
            }
            current_line.push(c);
            // Map every byte of this char to the current (line, col)
            let char_bytes = c.len_utf8();
            for i in 0..char_bytes {
                if byte_idx + i < mapping.len() {
                    mapping[byte_idx + i] = (line_idx, col_idx);
                }
            }
            byte_idx += char_bytes;
            current_width += c_width;
            col_idx += 1;
        }
        if !current_line.is_empty() {
            lines.push(current_line);
            line_idx += 1;
        }
        // Map the end of the text
        if byte_idx < mapping.len() {
            mapping[byte_idx] = (line_idx.saturating_sub(1), lines.last().map(|l| l.chars().count()).unwrap_or(0));
        }
        (lines, mapping)
    }

        /// Ensure the cursor index is always at a valid UTF-8 boundary and in bounds
        fn ensure_cursor_validity(&mut self) {
            if self.cursor_index > self.text.len() {
                self.cursor_index = self.text.len();
            }
            // Clamp to char boundary
            while self.cursor_index > 0 && !self.text.is_char_boundary(self.cursor_index) {
                self.cursor_index -= 1;
            }
        }
    pub fn new(x: f32, y: f32, width: f32, height: f32, font_size: f32) -> Self {
                Self {
                        preferred_col: None,
            x,
            y,
            width,
            height,
            text: String::new(),
            active: false,
            cursor_index: 0,
            cursor_timer: 0.0,
            cursor_visible: true,
            font_size,
            text_color: BLACK, // Default color for text
            border_color: DARKGRAY, // Default color for border
            background_color: LIGHTGRAY, // Default color for background
            cursor_color: BLACK, // Default color for cursor
            font: None, // Default to None (use system font)
            prompt: None, // Default to None (no prompt text)
            prompt_color: GRAY, // Default color for prompt text
            // Initialize key repeat values
            key_repeat_delay: 0.4, // 400ms initial delay before repeat
            key_repeat_rate: 0.05, // 50ms between repeats after initial delay
            key_repeat_timer: 0.0,
            last_key: None,
            enabled: true, // Default to enabled
            disabled_color: Color::new(0.7, 0.7, 0.7, 0.5), // Semi-transparent gray for disabled state
            multiline: false,
            max_chars: None,
            allowed_chars: None,
        }
    }

    /// Enable or disable multiline mode (wrapping within box)
    #[allow(unused)]
    pub fn set_multiline(&mut self, multiline: bool) -> &mut Self {
        self.multiline = multiline;
        self
    }
    #[allow(unused)]
    pub fn is_multiline(&self) -> bool {
        self.multiline
    }
    
    // Position and dimension getters/setters
    #[allow(unused)]
    pub fn get_x(&self) -> f32 {
        self.x
    }
    
    #[allow(unused)]
    pub fn set_x(&mut self, x: f32) -> &mut Self {
        self.x = x;
        self
    }
    
    #[allow(unused)]
    pub fn get_y(&self) -> f32 {
        self.y
    }
    
    #[allow(unused)]
    pub fn set_y(&mut self, y: f32) -> &mut Self {
        self.y = y;
        self
    }
    
    #[allow(unused)]
    pub fn get_width(&self) -> f32 {
        self.width
    }
    
    #[allow(unused)]
    pub fn set_width(&mut self, width: f32) -> &mut Self {
        self.width = width;
        self
    }
    
    #[allow(unused)]
    pub fn get_height(&self) -> f32 {
        self.height
    }
    
    #[allow(unused)]
    pub fn set_height(&mut self, height: f32) -> &mut Self {
        self.height = height;
        self
    }
    
    // Position convenience methods
    #[allow(unused)]
    pub fn get_position(&self) -> (f32, f32) {
        (self.x, self.y)
    }
    
    #[allow(unused)]
    pub fn set_position(&mut self, x: f32, y: f32) -> &mut Self {
        self.x = x;
        self.y = y;
        self
    }
    
    // Dimension convenience methods
    #[allow(unused)]
    pub fn get_dimensions(&self) -> (f32, f32) {
        (self.width, self.height)
    }
    
    #[allow(unused)]
    pub fn set_dimensions(&mut self, width: f32, height: f32) -> &mut Self {
        self.width = width;
        self.height = height;
        self
    }
    
    // Add a method to change colors
    #[allow(unused)]
    pub fn with_colors(&mut self, text_color: Color, border_color: Color, background_color: Color, cursor_color: Color) -> &mut Self {
        self.text_color = text_color;
        self.border_color = border_color;
        self.background_color = background_color;
        self.cursor_color = cursor_color;
        self
    }

    // Method to set custom font
    #[allow(unused)]
    pub fn with_font(&mut self, font: Font) -> &mut Self {
        self.font = Some(font);
        self
    }

    // Get the current text content
    #[allow(unused)]
    pub fn get_text(&self) -> String {
        self.text.clone()
    }
    
    // Set the text content - now accepts both String and &str
    #[allow(unused)]
    pub fn set_text<T: Into<String>>(&mut self, text: T) -> &mut Self {
        self.text = self.apply_text_constraints(&text.into());
        self.ensure_cursor_validity();
        self
    }
    
    // Active state getters/setters
    #[allow(unused)]
    pub fn is_active(&self) -> bool {
        self.active
    }

    #[allow(unused)]
    pub fn set_active(&mut self, active: bool) -> &mut Self {
        self.active = active;
        self
    }

    // Cursor index getters/setters
    #[allow(unused)]
    pub fn get_cursor_index(&self) -> usize {
        self.cursor_index
    }

    #[allow(unused)]
    pub fn set_cursor_index(&mut self, index: usize) -> &mut Self {
        if index <= self.text.len() {
            self.cursor_index = index;
        }
        self
    }

    // Font size getters/setters
    #[allow(unused)]
    pub fn get_font_size(&self) -> f32 {
        self.font_size
    }

    #[allow(unused)]
    pub fn set_font_size(&mut self, size: f32) -> &mut Self {
        self.font_size = size;
        self
    }

    // Color getters/setters
    #[allow(unused)]
    pub fn get_text_color(&self) -> Color {
        self.text_color
    }

    #[allow(unused)]
    pub fn set_text_color(&mut self, color: Color) -> &mut Self {
        self.text_color = color;
        self
    }

    #[allow(unused)]
    pub fn get_border_color(&self) -> Color {
        self.border_color
    }

    #[allow(unused)]
    pub fn set_border_color(&mut self, color: Color) -> &mut Self {
        self.border_color = color;
        self
    }

    #[allow(unused)]
    pub fn get_background_color(&self) -> Color {
        self.background_color
    }

    #[allow(unused)]
    pub fn set_background_color(&mut self, color: Color) -> &mut Self {
        self.background_color = color;
        self
    }

    #[allow(unused)]
    pub fn get_cursor_color(&self) -> Color {
        self.cursor_color
    }

    #[allow(unused)]
    pub fn set_cursor_color(&mut self, color: Color) -> &mut Self {
        self.cursor_color = color;
        self
    }

    // Font getter/setter
    #[allow(unused)]
    pub fn get_font(&self) -> Option<&Font> {
        self.font.as_ref()
    }

    // Prompt text getters/setters
    #[allow(unused)]
    pub fn get_prompt(&self) -> Option<&String> {
        self.prompt.as_ref()
    }

    #[allow(unused)]
    pub fn set_prompt<T: Into<String>>(&mut self, prompt: T) -> &mut Self {
        self.prompt = Some(prompt.into());
        self
    }

    #[allow(unused)]
    pub fn get_prompt_color(&self) -> Color {
        self.prompt_color
    }

    #[allow(unused)]
    pub fn set_prompt_color(&mut self, color: Color) -> &mut Self {
        self.prompt_color = color;
        self
    }

    // Key repeat settings getters/setters
    #[allow(unused)]
    pub fn get_key_repeat_delay(&self) -> f32 {
        self.key_repeat_delay
    }

    #[allow(unused)]
    pub fn set_key_repeat_delay(&mut self, delay: f32) -> &mut Self {
        self.key_repeat_delay = delay;
        self
    }

    #[allow(unused)]
    pub fn get_key_repeat_rate(&self) -> f32 {
        self.key_repeat_rate
    }

    #[allow(unused)]
    pub fn set_key_repeat_rate(&mut self, rate: f32) -> &mut Self {
        self.key_repeat_rate = rate;
        self
    }

    // Convenience method to set both key repeat values at once
    #[allow(unused)]
    pub fn with_key_repeat_settings(&mut self, delay: f32, rate: f32) -> &mut Self {
        self.key_repeat_delay = delay;
        self.key_repeat_rate = rate;
        self
    }

    // Enable/disable functionality
    #[allow(unused)]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[allow(unused)]
    pub fn set_enabled(&mut self, enabled: bool) -> &mut Self {
        self.enabled = enabled;
        if !enabled {
            self.active = false; // Deactivate if disabled
        }
        self
    }
    
    #[allow(unused)]
    pub fn get_disabled_color(&self) -> Color {
        self.disabled_color
    }
    
    #[allow(unused)]
    pub fn set_disabled_color(&mut self, color: Color) -> &mut Self {
        self.disabled_color = color;
        self
    }

    /// Set the maximum number of characters allowed in the text input.
    #[allow(unused)]
    pub fn set_max_chars(&mut self, max: usize) -> &mut Self {
        self.max_chars = Some(max);
        self
    }

    /// Remove the character limit (unlimited input).
    #[allow(unused)]
    pub fn clear_max_chars(&mut self) -> &mut Self {
        self.max_chars = None;
        self
    }

    /// Restrict text input to only the characters in the provided whitelist.
    #[allow(unused)]
    pub fn set_allowed_chars<T: Into<String>>(&mut self, allowed_chars: T) -> &mut Self {
        self.allowed_chars = Some(allowed_chars.into());
        self.text = self.apply_text_constraints(&self.text);
        self.ensure_cursor_validity();
        self
    }

    /// Remove any character whitelist and allow all characters again.
    #[allow(unused)]
    pub fn clear_allowed_chars(&mut self) -> &mut Self {
        self.allowed_chars = None;
        self
    }

    #[allow(unused)]
    pub fn get_allowed_chars(&self) -> Option<&str> {
        self.allowed_chars.as_deref()
    }

    // Primary method - both updates and draws the textbox
    #[allow(unused)]
    pub fn draw(&mut self) {
        self.update_internal();
        self.draw_internal();
    }
    
    // For cases when only drawing is needed without updating
    #[allow(unused)]
    pub fn draw_only(&self) {
        self.draw_internal();
    }
    
    // For cases when only updating is needed without drawing
    #[allow(unused)]
    pub fn update_only(&mut self) {
        self.update_internal();
    }

    // Now private - internal implementation only
    fn update_internal(&mut self) {
        // Skip all interaction if disabled
        if !self.enabled {
            self.active = false;
            self.cursor_visible = false;
            return;
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            self.active = mx >= self.x && mx <= self.x + self.width && my >= self.y && my <= self.y + self.height;

            if self.active {
                let text_x = self.x + 5.0;
                let text_y = self.y + 5.0;
                let mouse_x = mx - text_x;
                let mouse_y = my - text_y;
                let (wrapped_lines, mapping) = self.get_wrapped_lines_and_mapping();
                // Determine which line was clicked
                let line_height = self.font_size + 2.0;
                let clicked_line = (mouse_y / line_height).floor() as usize;
                let clicked_line = clicked_line.min(wrapped_lines.len().saturating_sub(1));
                // Now, find the closest column in that line
                let mut col = 0;
                let mut x_offset = 0.0;
                let font = self.font.as_ref();
                if clicked_line < wrapped_lines.len() {
                    let line = &wrapped_lines[clicked_line];
                    for (i, c) in line.chars().enumerate() {
                        let c_width = measure_text(&c.to_string(), font, self.font_size as u16, 1.0).width;
                        if x_offset + c_width / 2.0 > mouse_x {
                            break;
                        }
                        x_offset += c_width;
                        col = i + 1;
                    }
                }
                // Find the last byte index in mapping that matches (clicked_line, col)
                let mut last_match = None;
                for (byte_idx, &(line, ccol)) in mapping.iter().enumerate() {
                    if line == clicked_line && ccol == col {
                        last_match = Some(byte_idx);
                    }
                }
                if let Some(byte_idx) = last_match {
                    self.cursor_index = byte_idx;
                } else {
                    self.cursor_index = self.text.len();
                }
                self.ensure_cursor_validity();
            }
        }
    
        if self.active {
            // Handle typing
            while let Some(c) = get_char_pressed() {
                if !c.is_control() && self.can_insert_char(c) {
                    self.text.insert(self.cursor_index, c);
                    self.cursor_index += c.len_utf8();
                    self.ensure_cursor_validity();
                }
            }
            // Handle Enter key for multiline
            if self.multiline && is_key_pressed(KeyCode::Enter) && self.can_insert_char('\n') {
                self.text.insert(self.cursor_index, '\n');
                self.cursor_index += 1;
                self.ensure_cursor_validity();
            }

            // Initial key presses
            let key_delete_pressed = is_key_pressed(KeyCode::Delete);
            let key_backspace_pressed = is_key_pressed(KeyCode::Backspace);
            let key_left_pressed = is_key_pressed(KeyCode::Left);
            let key_right_pressed = is_key_pressed(KeyCode::Right);
            let key_up_pressed = is_key_pressed(KeyCode::Up);
            let key_down_pressed = is_key_pressed(KeyCode::Down);

            // Handle initial key presses
            if key_delete_pressed && self.cursor_index < self.text.len() {
                if let Some((_, c)) = self.text[self.cursor_index..].char_indices().next() {
                    let char_len = c.len_utf8();
                    self.text.replace_range(self.cursor_index..self.cursor_index + char_len, "");
                    self.ensure_cursor_validity();
                }
                self.last_key = Some(KeyCode::Delete);
                self.key_repeat_timer = 0.0;
            } else if key_backspace_pressed && self.cursor_index > 0 {
                if let Some((prev_offset, _c)) = self.text[..self.cursor_index].char_indices().rev().next() {
                    self.text.replace_range(prev_offset..self.cursor_index, "");
                    self.cursor_index = prev_offset;
                    self.ensure_cursor_validity();
                }
                self.last_key = Some(KeyCode::Backspace);
                self.key_repeat_timer = 0.0;
            } else if key_left_pressed && self.cursor_index > 0 {
                let prev_char = self.text[..self.cursor_index].chars().last().unwrap();
                let char_len = prev_char.len_utf8();
                self.cursor_index -= char_len;
                self.ensure_cursor_validity();
                self.last_key = Some(KeyCode::Left);
                self.key_repeat_timer = 0.0;
                self.preferred_col = None;
            } else if key_right_pressed && self.cursor_index < self.text.len() {
                let next_char = self.text[self.cursor_index..].chars().next().unwrap();
                let char_len = next_char.len_utf8();
                self.cursor_index += char_len;
                self.ensure_cursor_validity();
                self.last_key = Some(KeyCode::Right);
                self.key_repeat_timer = 0.0;
                self.preferred_col = None;
            } else if self.multiline && (key_up_pressed || key_down_pressed) {
                // Robust multiline up/down navigation using mapping
                let (wrapped_lines, mapping) = self.get_wrapped_lines_and_mapping();
                let cursor_idx = self.cursor_index.min(self.text.len());
                let (cur_line, cur_col) = if cursor_idx < mapping.len() {
                    mapping[cursor_idx]
                } else {
                    (wrapped_lines.len().saturating_sub(1), wrapped_lines.last().map(|l| l.chars().count()).unwrap_or(0))
                };
                // Store preferred_col for vertical navigation
                if self.preferred_col.is_none() {
                    self.preferred_col = Some(cur_col);
                }
                let mut new_line = cur_line;
                if key_up_pressed && cur_line > 0 {
                    new_line = cur_line - 1;
                } else if key_down_pressed && cur_line + 1 < wrapped_lines.len() {
                    new_line = cur_line + 1;
                }
                if new_line != cur_line {
                    let preferred_col = self.preferred_col.unwrap_or(cur_col);
                    let new_line_len = wrapped_lines[new_line].chars().count();
                    let new_col = preferred_col.min(new_line_len);
                    // Find the last byte index in mapping that matches (new_line, new_col)
                    let mut last_match = None;
                    for (byte_idx, &(line, col)) in mapping.iter().enumerate() {
                        if line == new_line && col == new_col {
                            last_match = Some(byte_idx);
                        }
                    }
                    if let Some(byte_idx) = last_match {
                        self.cursor_index = byte_idx;
                        self.ensure_cursor_validity();
                    } else {
                        self.cursor_index = self.text.len();
                        self.ensure_cursor_validity();
                    }
                }
                // Reset preferred_col if left/right or typing
                if key_left_pressed || key_right_pressed || get_char_pressed().is_some() {
                    self.preferred_col = None;
                }
            }

            // Handle key repeat functionality
            if let Some(key) = self.last_key {
                if is_key_down(key) {
                    self.key_repeat_timer += get_frame_time();
                    if self.key_repeat_timer >= self.key_repeat_delay {
                        self.key_repeat_timer -= self.key_repeat_rate;
                        match key {
                            KeyCode::Left => {
                                if self.cursor_index > 0 {
                                    let prev_char = self.text[..self.cursor_index].chars().last().unwrap();
                                    let char_len = prev_char.len_utf8();
                                    self.cursor_index -= char_len;
                                    self.ensure_cursor_validity();
                                }
                            }
                            KeyCode::Right => {
                                if self.cursor_index < self.text.len() {
                                    let next_char = self.text[self.cursor_index..].chars().next().unwrap();
                                    let char_len = next_char.len_utf8();
                                    self.cursor_index += char_len;
                                    self.ensure_cursor_validity();
                                }
                            }
                            KeyCode::Delete => {
                                if self.cursor_index < self.text.len() {
                                    if let Some((_, c)) = self.text[self.cursor_index..].char_indices().next() {
                                        let char_len = c.len_utf8();
                                        self.text.replace_range(self.cursor_index..self.cursor_index + char_len, "");
                                        self.ensure_cursor_validity();
                                    }
                                }
                            }
                            KeyCode::Backspace => {
                                if self.cursor_index > 0 {
                                    if let Some((prev_offset, _c)) = self.text[..self.cursor_index].char_indices().rev().next() {
                                        self.text.replace_range(prev_offset..self.cursor_index, "");
                                        self.cursor_index = prev_offset;
                                        self.ensure_cursor_validity();
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                } else {
                    self.last_key = None;
                    self.key_repeat_timer = 0.0;
                }
            }

            self.cursor_timer += get_frame_time();
            if self.cursor_timer >= 0.5 {
                self.cursor_visible = !self.cursor_visible;
                self.cursor_timer = 0.0;
            }
        } else {
            self.cursor_visible = false;
        }
    }

    
    // Now private - internal implementation only
    fn draw_internal(&self) {
        let padding = 5.0;
        let text_x = self.x + padding;
        let text_y = self.y + self.font_size + padding;

        // Draw the background with customizable colors (or disabled color when disabled)
        if self.enabled {
            draw_rectangle(self.x, self.y, self.width, self.height, self.background_color);
        } else {
            draw_rectangle(self.x, self.y, self.width, self.height, self.disabled_color);
        }

        let text_color = if self.enabled { self.text_color } else { GRAY };
        let prompt_color = if self.enabled { self.prompt_color } else { GRAY };

        // Draw text (with wrapping if multiline)
        if self.text.is_empty() {
            if let Some(prompt) = &self.prompt {
                match &self.font {
                    Some(font) => {
                        draw_text_ex(
                            prompt,
                            text_x,
                            text_y,
                            TextParams {
                                font: Some(font),
                                font_size: self.font_size as u16,
                                color: prompt_color,
                                ..Default::default()
                            },
                        );
                    },
                    None => {
                        draw_text(prompt, text_x, text_y, self.font_size, prompt_color);
                    }
                }
            }
        } else {
            let (wrapped_lines, _mapping) = self.get_wrapped_lines_and_mapping();
            for (i, line) in wrapped_lines.iter().enumerate() {
                let y = text_y + i as f32 * (self.font_size + 2.0);
                match &self.font {
                    Some(font) => {
                        draw_text_ex(
                            line,
                            text_x,
                            y,
                            TextParams {
                                font: Some(font),
                                font_size: self.font_size as u16,
                                color: text_color,
                                ..Default::default()
                            },
                        );
                    },
                    None => {
                        draw_text(line, text_x, y, self.font_size, text_color);
                    }
                }
            }
        }

        // Cursor rendering for multiline (basic support)
        if self.enabled && self.active && self.cursor_visible {
            if self.multiline {
                // Use the same mapping as navigation for accurate cursor placement
                let (wrapped_lines, mapping) = self.get_wrapped_lines_and_mapping();
                let cursor_idx = self.cursor_index.min(self.text.len());
                let (cursor_line, cursor_col) = if cursor_idx < mapping.len() {
                    mapping[cursor_idx]
                } else {
                    (wrapped_lines.len().saturating_sub(1), wrapped_lines.last().map(|l| l.chars().count()).unwrap_or(0))
                };
                let mut cursor_offset = 0.0;
                let font = self.font.as_ref();
                if cursor_line < wrapped_lines.len() {
                    let line = &wrapped_lines[cursor_line];
                    for c in line.chars().take(cursor_col) {
                        cursor_offset += measure_text(&c.to_string(), font, self.font_size as u16, 1.0).width;
                    }
                }
                let cursor_spacing = 2.0;
                let y = text_y + cursor_line as f32 * (self.font_size + 2.0);
                draw_line(
                    text_x + cursor_offset + cursor_spacing,
                    y - self.font_size * 0.7,
                    text_x + cursor_offset + cursor_spacing,
                    y + 2.0,
                    1.0,
                    self.cursor_color,
                );
            } else {
                let mut cursor_offset = 0.0;
                if self.cursor_index > 0 {
                    let cursor_text = &self.text[..self.cursor_index];
                    if let Some(font) = &self.font {
                        for c in cursor_text.chars() {
                            cursor_offset += measure_text(&c.to_string(), Some(font), self.font_size as u16, 1.0).width;
                        }
                    } else {
                        for c in cursor_text.chars() {
                            cursor_offset += measure_text(&c.to_string(), None, self.font_size as u16, 1.0).width;
                        }
                    }
                }
                let cursor_spacing = 2.0;
                draw_line(
                    text_x + cursor_offset + cursor_spacing,
                    text_y - self.font_size * 0.7,
                    text_x + cursor_offset + cursor_spacing,
                    text_y + 2.0,
                    1.0,
                    self.cursor_color,
                );
            }
        }

        let border_color = if self.enabled { self.border_color } else { GRAY };
        draw_rectangle_lines(self.x, self.y, self.width, self.height, 2.0, border_color);
    }
}


