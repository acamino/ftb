use std::fmt;
use unicode_width::UnicodeWidthStr;

/// Errors that can occur during table formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableError {
    /// Table must have at least a header row and separator row
    MissingSeparator,

    /// Table exceeds maximum allowed size
    TableTooLarge {
        rows: usize,
        cols: usize,
        max_rows: usize,
        max_cols: usize,
    },

    /// Invalid table structure
    InvalidStructure(String),

    /// Input is empty or contains no table
    EmptyInput,
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TableError::MissingSeparator => {
                write!(f, "Table must have at least a header and separator row")
            }
            TableError::TableTooLarge {
                rows,
                cols,
                max_rows,
                max_cols,
            } => {
                write!(
                    f,
                    "Table too large: {rows}×{cols} cells (max {max_rows}×{max_cols})"
                )
            }
            TableError::InvalidStructure(msg) => {
                write!(f, "Invalid table structure: {msg}")
            }
            TableError::EmptyInput => {
                write!(f, "Input is empty or contains no table")
            }
        }
    }
}

impl std::error::Error for TableError {}

/// Result type for table formatting operations.
pub type Result<T> = std::result::Result<T, TableError>;

/// A Markdown table formatter that aligns columns properly.
///
/// This is a port of the JavaScript formatter from <http://markdowntable.com/>
pub struct TableFormatter {
    cells: Vec<Vec<String>>,
    column_widths: Vec<usize>,
}

impl TableFormatter {
    /// Creates a new `TableFormatter` instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            column_widths: Vec::new(),
        }
    }

    /// Formats tables within a Markdown document, preserving all other content.
    ///
    /// This method scans the document for tables and formats them in place while
    /// preserving all surrounding text, code blocks, lists, and other Markdown elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use ftb::TableFormatter;
    ///
    /// let doc = "# Title\n\nSome text.\n\n| a | b |\n|-|-|\n| 1 | 2 |\n\nMore text.";
    /// let mut formatter = TableFormatter::new();
    /// let output = formatter.format_document(doc);
    /// assert!(output.contains("# Title"));
    /// assert!(output.contains("Some text"));
    /// assert!(output.contains("| a | b |"));
    /// ```
    pub fn format_document(&mut self, document: &str) -> String {
        let mut output = String::with_capacity(document.len() + 1024);
        let lines: Vec<&str> = document.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // Check if this line starts a table (contains '|')
            if line.trim().starts_with('|')
                || (line.contains('|') && !line.trim().starts_with("```"))
            {
                // Try to extract and format the table
                if let Some((table_lines, formatted)) = self.try_format_table_at(&lines, i) {
                    output.push_str(&formatted);
                    i += table_lines;
                    continue;
                }
            }

            // Not a table or table formatting failed - preserve the line as-is
            output.push_str(line);
            output.push('\n');
            i += 1;
        }

        // Remove trailing newline if original didn't have one
        if !document.ends_with('\n') && output.ends_with('\n') {
            output.pop();
        }

        output
    }

    /// Attempts to extract and format a table starting at the given line index.
    ///
    /// Returns `Some((num_lines, formatted_table))` if a valid table was found and formatted,
    /// where `num_lines` is the number of lines consumed from the input.
    fn try_format_table_at(&mut self, lines: &[&str], start: usize) -> Option<(usize, String)> {
        if start >= lines.len() {
            return None;
        }

        // Scan forward to find the end of the table
        let mut end = start;
        while end < lines.len() && lines[end].contains('|') {
            end += 1;
        }

        if end == start {
            return None;
        }

        // Extract table lines
        let table_text = lines[start..end].join("\n");

        // Try to format the table
        match self.format_table(&table_text) {
            Ok(formatted) => {
                let num_lines = end - start;
                Some((num_lines, formatted))
            }
            Err(_) => {
                // If formatting fails, this might not be a valid table
                // Return None to preserve it as-is
                None
            }
        }
    }

    /// Formats a markdown table string, returning the aligned version.
    ///
    /// # Errors
    ///
    /// Returns `TableError` if:
    /// - Input is empty or contains no table
    /// - Table has fewer than 2 rows (header + separator minimum)
    /// - Table exceeds maximum size limits
    /// - Table structure is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use ftb::TableFormatter;
    ///
    /// let mut formatter = TableFormatter::new();
    /// let result = formatter.format_table("| a | b |\n|-|-|\n| 1 | 2 |");
    /// assert!(result.is_ok());
    /// ```
    pub fn format_table(&mut self, table: &str) -> Result<String> {
        const MAX_ROWS: usize = 100_000;
        const MAX_COLS: usize = 1_000;
        const MAX_CELLS: usize = 1_000_000;

        // Reset state to allow formatter reuse
        self.cells.clear();
        self.column_widths.clear();

        // Check for empty input
        if table.trim().is_empty() {
            return Err(TableError::EmptyInput);
        }

        // Import and parse table
        self.import_table(table)?;

        // Validate minimum structure
        if self.cells.len() < 2 {
            return Err(TableError::MissingSeparator);
        }

        // Validate maximum size
        let num_rows = self.cells.len();
        let num_cols = self.column_widths.len();
        let total_cells = num_rows * num_cols;

        if num_rows > MAX_ROWS || num_cols > MAX_COLS || total_cells > MAX_CELLS {
            return Err(TableError::TableTooLarge {
                rows: num_rows,
                cols: num_cols,
                max_rows: MAX_ROWS,
                max_cols: MAX_COLS,
            });
        }

        // Validate separator row (row 1)
        if !self.is_separator_row(1) {
            return Err(TableError::InvalidStructure(
                "Row 2 is not a valid separator row".to_string(),
            ));
        }

        // Process table
        self.get_column_widths();
        self.add_missing_cell_columns();
        self.pad_cells_for_output();

        // Render output
        Ok(self.render_output())
    }

    /// Checks if a row is a valid separator row (all cells are dashes).
    fn is_separator_row(&self, row_index: usize) -> bool {
        if row_index >= self.cells.len() {
            return false;
        }

        let row = &self.cells[row_index];
        if row.is_empty() {
            return false;
        }

        row.iter()
            .all(|cell| !cell.is_empty() && cell.chars().all(|c| c == '-'))
    }

    /// Renders the formatted table to a string.
    fn render_output(&self) -> String {
        let num_columns = self.column_widths.len();
        let total_width: usize = self.column_widths.iter().sum();

        // Calculate more accurate capacity
        // Header: "| " + content + " | " per cell + "\n"
        let header_len = 2 + total_width + (num_columns.saturating_sub(1)) * 3 + 2 + 1;
        // Separator: same structure but all dashes
        let sep_len = header_len;
        // Data rows: same as header
        let data_rows_len = self.cells.len().saturating_sub(2) * header_len;

        let estimated_capacity = header_len + sep_len + data_rows_len;
        let mut output = String::with_capacity(estimated_capacity);

        // Header
        if !self.cells.is_empty() {
            output.push_str("| ");
            output.push_str(&self.cells[0].join(" | "));
            output.push_str(" |\n");
        }

        // Separator
        if self.cells.len() > 1 {
            output.push_str("|-");
            output.push_str(&self.cells[1].join("-|-"));
            output.push_str("-|\n");
        }

        // Data rows
        for row in self.cells.iter().skip(2) {
            output.push_str("| ");
            output.push_str(&row.join(" | "));
            output.push_str(" |\n");
        }

        output
    }

    /// Imports and parses the table string into cells.
    fn import_table(&mut self, table: &str) -> Result<()> {
        // Skip leading empty lines (O(n) instead of O(n²))
        let table_rows: Vec<&str> = table
            .lines()
            .skip_while(|line| !line.contains('|'))
            .collect();

        if table_rows.is_empty() {
            return Err(TableError::InvalidStructure(
                "No table rows found in input".to_string(),
            ));
        }

        for (row_i, row) in table_rows.iter().enumerate() {
            // Skip empty lines
            if !row.contains('|') {
                continue;
            }

            let row_columns: Vec<String> = row
                .split('|')
                .map(|cell| {
                    let mut trimmed = cell.trim().to_string();

                    // If it's the separator row, normalize to single dash
                    if row_i == 1 && trimmed.chars().all(|c| c == '-') && !trimmed.is_empty() {
                        trimmed = "-".to_string();
                    }

                    trimmed
                })
                .collect();

            self.cells.push(row_columns);
        }

        // Remove leading and trailing empty columns
        self.remove_empty_edge_columns();

        Ok(())
    }

    /// Removes empty leading and trailing columns.
    fn remove_empty_edge_columns(&mut self) {
        // Calculate widths once
        self.get_column_widths();

        if self.column_widths.is_empty() {
            return;
        }

        let mut removed_leading = false;
        let mut removed_trailing = false;

        // Remove leading empty column if exists
        if self.column_widths[0] == 0 {
            for row in &mut self.cells {
                if !row.is_empty() {
                    row.remove(0);
                }
            }
            removed_leading = true;
        }

        // Check trailing column (use original length if we removed leading)
        if removed_leading {
            self.get_column_widths();
        }

        if !self.column_widths.is_empty() {
            let last_idx = self.column_widths.len() - 1;
            if self.column_widths[last_idx] == 0 {
                for row in &mut self.cells {
                    if row.len() == self.column_widths.len() {
                        row.pop();
                    }
                }
                removed_trailing = true;
            }
        }

        // Only recalculate if we removed trailing column
        if removed_trailing {
            self.get_column_widths();
        }
    }

    /// Calculates the maximum width needed for each column.
    fn get_column_widths(&mut self) {
        self.column_widths.clear();

        for row in &self.cells {
            for (col_i, cell) in row.iter().enumerate() {
                let cell_width = cell.width();
                if col_i >= self.column_widths.len() {
                    self.column_widths.push(cell_width);
                } else if self.column_widths[col_i] < cell_width {
                    self.column_widths[col_i] = cell_width;
                }
            }
        }
    }

    /// Adds missing cells to rows that don't have enough columns.
    fn add_missing_cell_columns(&mut self) {
        let num_columns = self.column_widths.len();

        for row in &mut self.cells {
            while row.len() < num_columns {
                row.push(String::new());
            }
        }
    }

    /// Pads cells with spaces (or dashes for separator row) to align columns.
    fn pad_cells_for_output(&mut self) {
        for (row_i, row) in self.cells.iter_mut().enumerate() {
            for (col_i, cell) in row.iter_mut().enumerate() {
                let target_width = self.column_widths[col_i];
                let current_width = cell.width();
                let padding = target_width.saturating_sub(current_width);

                if padding == 0 {
                    continue;
                }

                // Handle the separator row
                if row_i == 1 {
                    // Use iterator instead of String::repeat (avoids allocation)
                    cell.extend(std::iter::repeat('-').take(padding));
                }
                // Handle anything that's not the separator row
                else {
                    cell.extend(std::iter::repeat(' ').take(padding));
                }
            }
        }
    }
}

impl Default for TableFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_table() {
        let input = "| h1 | h2 | h3 |\n|-|-|-|\n| data1 | data2 | data3 |\n";
        let mut formatter = TableFormatter::new();
        let output = formatter
            .format_table(input)
            .expect("Should format successfully");

        assert!(output.contains("| h1    | h2    | h3    |"));
        assert!(output.contains("| data1 | data2 | data3 |"));
    }

    #[test]
    fn test_irregular_columns() {
        let input = "h1 | h2 | h3\n-|-|-\ndata-1 | data-2 | data-3";
        let mut formatter = TableFormatter::new();
        let output = formatter
            .format_table(input)
            .expect("Should format successfully");

        assert!(output.contains("h1"));
        assert!(output.contains("h2"));
        assert!(output.contains("h3"));
        assert!(output.contains("data-1"));
    }

    #[test]
    fn test_missing_cells() {
        let input = "| Header 1 | Header 2 | Header 3 |\n|----|---|-|\n| data1a | Data is longer | 1 |\n| d1b | add a cell|";
        let mut formatter = TableFormatter::new();
        let output = formatter
            .format_table(input)
            .expect("Should format successfully");

        // Should have 3 columns
        let lines: Vec<&str> = output.lines().collect();
        for line in lines {
            let pipe_count = line.matches('|').count();
            assert_eq!(pipe_count, 4); // 4 pipes for 3 columns
        }
    }

    #[test]
    fn test_empty_table() {
        let input = "";
        let mut formatter = TableFormatter::new();
        let result = formatter.format_table(input);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TableError::EmptyInput));
    }

    #[test]
    fn test_column_width_calculation() {
        let mut formatter = TableFormatter::new();
        formatter.cells = vec![
            vec!["a".to_string(), "bb".to_string()],
            vec!["ccc".to_string(), "d".to_string()],
        ];
        formatter.get_column_widths();

        assert_eq!(formatter.column_widths, vec![3, 2]);
    }

    #[test]
    fn test_unicode_emoji() {
        let input = "| Emoji | Description |\n|-|-|\n| 😀 | smile |\n| 🎉 | party |\n";
        let mut formatter = TableFormatter::new();
        let output = formatter
            .format_table(input)
            .expect("Should format successfully");

        // Check output structure
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 4); // header + separator + 2 data rows

        // All lines should have 3 pipes (2 columns)
        for line in &lines {
            assert_eq!(line.matches('|').count(), 3, "Line: {}", line);
        }

        // Check that content is properly aligned
        assert!(output.contains("😀"));
        assert!(output.contains("🎉"));
    }

    #[test]
    fn test_unicode_cjk_characters() {
        let input = "| English | 中文 | 日本語 |\n|-|-|-|\n| hello | 你好 | こんにちは |\n";
        let mut formatter = TableFormatter::new();
        let output = formatter
            .format_table(input)
            .expect("Should format successfully");

        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 3);

        // All lines should have 4 pipes (3 columns)
        for line in &lines {
            assert_eq!(line.matches('|').count(), 4, "Line: {}", line);
        }

        // Check that CJK characters are present
        assert!(output.contains("中文"));
        assert!(output.contains("日本語"));
        assert!(output.contains("你好"));
        assert!(output.contains("こんにちは"));
    }

    #[test]
    fn test_unicode_accented_characters() {
        let input = "| Name | País |\n|-|-|\n| José | España |\n| François | Françe |\n";
        let mut formatter = TableFormatter::new();
        let output = formatter
            .format_table(input)
            .expect("Should format successfully");

        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 4);

        // Check accented characters are preserved
        assert!(output.contains("José"));
        assert!(output.contains("España"));
        assert!(output.contains("François"));
        assert!(output.contains("Françe"));
    }

    #[test]
    fn test_mixed_unicode_and_ascii() {
        let input = "| ID | Name | Status |\n|-|-|-|\n| 1 | 田中 | ✓ |\n| 2 | John | ✗ |\n";
        let mut formatter = TableFormatter::new();
        let output = formatter
            .format_table(input)
            .expect("Should format successfully");

        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 4);

        for line in &lines {
            assert_eq!(line.matches('|').count(), 4);
        }

        assert!(output.contains("田中"));
        assert!(output.contains("✓"));
        assert!(output.contains("✗"));
    }

    #[test]
    fn test_formatter_reusability() {
        let mut formatter = TableFormatter::new();

        // Format first table
        let input1 = "| h1 | h2 |\n|-|-|\n| a | b |\n";
        let output1 = formatter
            .format_table(input1)
            .expect("Should format successfully");
        assert!(output1.contains("h1"));
        assert!(output1.contains("h2"));

        // Format second table with the same formatter
        let input2 = "| col1 | col2 | col3 |\n|-|-|-|\n| x | y | z |\n";
        let output2 = formatter
            .format_table(input2)
            .expect("Should format successfully");
        assert!(output2.contains("col1"));
        assert!(output2.contains("col2"));
        assert!(output2.contains("col3"));

        // Verify the second output doesn't contain data from first table
        let lines2: Vec<&str> = output2.lines().collect();
        assert_eq!(lines2[0].matches('|').count(), 4); // 3 columns = 4 pipes
    }

    #[test]
    fn test_formatter_reusability_different_sizes() {
        let mut formatter = TableFormatter::new();

        // Format a large table
        let input1 = "| a | b | c | d |\n|-|-|-|-|\n| 1 | 2 | 3 | 4 |\n| 5 | 6 | 7 | 8 |\n";
        formatter
            .format_table(input1)
            .expect("Should format successfully");

        // Format a smaller table - should work correctly
        let input2 = "| x | y |\n|-|-|\n| 9 | 10 |\n";
        let output2 = formatter
            .format_table(input2)
            .expect("Should format successfully");

        let lines2: Vec<&str> = output2.lines().collect();
        assert_eq!(lines2.len(), 3);
        // Should have 2 columns (3 pipes)
        for line in &lines2 {
            assert_eq!(line.matches('|').count(), 3, "Line: {}", line);
        }
    }

    // New error handling tests
    #[test]
    fn test_missing_separator_error() {
        let input = "| h1 | h2 |\n| d1 | d2 |";
        let mut formatter = TableFormatter::new();
        let result = formatter.format_table(input);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TableError::InvalidStructure(_)
        ));
    }

    #[test]
    fn test_single_row_error() {
        let input = "| header |";
        let mut formatter = TableFormatter::new();
        let result = formatter.format_table(input);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TableError::MissingSeparator));
    }

    #[test]
    fn test_whitespace_only_error() {
        let input = "   \n\n   \t  \n";
        let mut formatter = TableFormatter::new();
        let result = formatter.format_table(input);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TableError::EmptyInput));
    }

    #[test]
    fn test_format_after_error() {
        let mut formatter = TableFormatter::new();

        // First call fails
        let result1 = formatter.format_table("invalid");
        assert!(result1.is_err());

        // Second call should work
        let result2 = formatter.format_table("| h |\n|-|\n| d |");
        assert!(result2.is_ok());
        let output = result2.unwrap();
        assert!(output.contains("| h |"));
        assert!(output.contains("| d |"));
    }

    // Document formatting tests
    #[test]
    fn test_format_document_with_single_table() {
        let input =
            "# Header\n\nSome text before.\n\n| a | b |\n|-|-|\n| 1 | 2 |\n\nSome text after.";
        let mut formatter = TableFormatter::new();
        let output = formatter.format_document(input);

        assert!(output.contains("# Header"));
        assert!(output.contains("Some text before"));
        assert!(output.contains("Some text after"));
        assert!(output.contains("| a | b |"));
        assert!(output.contains("| 1 | 2 |"));
    }

    #[test]
    fn test_format_document_with_multiple_tables() {
        let input = "Table 1:\n\n| a | b |\n|-|-|\n| 1 | 2 |\n\nTable 2:\n\n| x | y | z |\n|-|-|-|\n| 3 | 4 | 5 |";
        let mut formatter = TableFormatter::new();
        let output = formatter.format_document(input);

        assert!(output.contains("Table 1:"));
        assert!(output.contains("Table 2:"));
        assert!(output.contains("| a | b |"));
        assert!(output.contains("| x | y | z |"));
    }

    #[test]
    fn test_format_document_preserves_code_blocks() {
        let input = "```\n| fake | table |\n```\n\n| real | table |\n|-|-|\n| a | b |";
        let mut formatter = TableFormatter::new();
        let output = formatter.format_document(input);

        // Code block should be preserved as-is
        assert!(output.contains("```"));
        assert!(output.contains("| fake | table |"));
        // Real table should be formatted (checking for formatted structure)
        assert!(output.contains("| real"));
        assert!(output.contains("| a"));
    }

    #[test]
    fn test_format_document_with_no_tables() {
        let input = "# Title\n\nJust some text.\n\nNo tables here.";
        let mut formatter = TableFormatter::new();
        let output = formatter.format_document(input);

        // Should be unchanged
        assert_eq!(output, input);
    }

    #[test]
    fn test_format_document_with_malformed_table() {
        let input = "# Title\n\n| incomplete | table\n\nMore text.";
        let mut formatter = TableFormatter::new();
        let output = formatter.format_document(input);

        // Malformed table should be preserved as-is
        assert!(output.contains("| incomplete | table"));
        assert!(output.contains("More text."));
    }
}
