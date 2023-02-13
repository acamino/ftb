use unicode_width::UnicodeWidthStr;

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

    /// Formats a markdown table string, returning the aligned version.
    pub fn format_table(&mut self, table: &str) -> String {
        // Reset state to allow formatter reuse
        self.cells.clear();
        self.column_widths.clear();

        self.import_table(table);
        self.get_column_widths();
        self.add_missing_cell_columns();
        self.pad_cells_for_output();

        // Pre-allocate output buffer based on table dimensions
        let num_columns = self.column_widths.len();
        let total_width: usize = self.column_widths.iter().sum();
        // Estimate: width + separators (3 per column) + newlines
        let estimated_capacity = self.cells.len() * (total_width + num_columns * 3 + 2);
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
    fn import_table(&mut self, table: &str) {
        let mut table_rows: Vec<&str> = table.lines().collect();

        // Remove leading empty lines
        while !table_rows.is_empty() && !table_rows[0].contains('|') {
            table_rows.remove(0);
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

                    // If it's the separator row, parse down the dashes
                    if row_i == 1 && trimmed.chars().all(|c| c == '-') && !trimmed.is_empty() {
                        trimmed = "-".to_string();
                    }

                    trimmed
                })
                .collect();

            self.cells.push(row_columns);
        }

        // Remove leading and trailing columns if they are empty
        self.get_column_widths();

        if !self.column_widths.is_empty() && self.column_widths[0] == 0 {
            for row in &mut self.cells {
                if !row.is_empty() {
                    row.remove(0);
                }
            }
        }

        self.get_column_widths();

        // Check to see if the last item in column widths is empty
        if !self.column_widths.is_empty() {
            let last_width = self.column_widths[self.column_widths.len() - 1];
            if last_width == 0 {
                for row in &mut self.cells {
                    // Only remove the row if it is in the proper last slot
                    if row.len() == self.column_widths.len() {
                        row.pop();
                    }
                }
            }
        }

        self.get_column_widths();
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

                // Handle the separator row
                if row_i == 1 {
                    let padding = target_width.saturating_sub(current_width);
                    cell.push_str(&"-".repeat(padding));
                }
                // Handle anything that's not the separator row
                else {
                    let padding = target_width.saturating_sub(current_width);
                    cell.push_str(&" ".repeat(padding));
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
        let output = formatter.format_table(input);

        assert!(output.contains("| h1    | h2    | h3    |"));
        assert!(output.contains("| data1 | data2 | data3 |"));
    }

    #[test]
    fn test_irregular_columns() {
        let input = "h1 | h2 | h3\n-|-|-\ndata-1 | data-2 | data-3";
        let mut formatter = TableFormatter::new();
        let output = formatter.format_table(input);

        assert!(output.contains("h1"));
        assert!(output.contains("h2"));
        assert!(output.contains("h3"));
        assert!(output.contains("data-1"));
    }

    #[test]
    fn test_missing_cells() {
        let input = "| Header 1 | Header 2 | Header 3 |\n|----|---|-|\n| data1a | Data is longer | 1 |\n| d1b | add a cell|";
        let mut formatter = TableFormatter::new();
        let output = formatter.format_table(input);

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
        let output = formatter.format_table(input);

        assert_eq!(output, "");
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
        let output = formatter.format_table(input);

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
        let output = formatter.format_table(input);

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
        let output = formatter.format_table(input);

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
        let output = formatter.format_table(input);

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
        let output1 = formatter.format_table(input1);
        assert!(output1.contains("h1"));
        assert!(output1.contains("h2"));

        // Format second table with the same formatter
        let input2 = "| col1 | col2 | col3 |\n|-|-|-|\n| x | y | z |\n";
        let output2 = formatter.format_table(input2);
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
        formatter.format_table(input1);

        // Format a smaller table - should work correctly
        let input2 = "| x | y |\n|-|-|\n| 9 | 10 |\n";
        let output2 = formatter.format_table(input2);

        let lines2: Vec<&str> = output2.lines().collect();
        assert_eq!(lines2.len(), 3);
        // Should have 2 columns (3 pipes)
        for line in &lines2 {
            assert_eq!(line.matches('|').count(), 3, "Line: {}", line);
        }
    }
}
