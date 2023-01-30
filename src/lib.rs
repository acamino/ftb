/// A Markdown table formatter that aligns columns properly.
///
/// This is a port of the JavaScript formatter from http://markdowntable.com/
pub struct TableFormatter {
    cells: Vec<Vec<String>>,
    column_widths: Vec<usize>,
}

impl TableFormatter {
    /// Creates a new TableFormatter instance.
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            column_widths: Vec::new(),
        }
    }

    /// Formats a markdown table string, returning the aligned version.
    pub fn format_table(&mut self, table: &str) -> String {
        self.import_table(table);
        self.get_column_widths();
        self.add_missing_cell_columns();
        self.pad_cells_for_output();

        let mut output = String::new();

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
                    if row_i == 1 {
                        trimmed = trimmed.replace(|c: char| c == '-', "-");
                        if trimmed.chars().all(|c| c == '-') && !trimmed.is_empty() {
                            trimmed = "-".to_string();
                        }
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
                if col_i >= self.column_widths.len() {
                    self.column_widths.push(cell.len());
                } else if self.column_widths[col_i] < cell.len() {
                    self.column_widths[col_i] = cell.len();
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

                // Handle anything that's not the separator row
                if row_i != 1 {
                    while cell.len() < target_width {
                        cell.push(' ');
                    }
                }
                // Handle the separator row
                else {
                    while cell.len() < target_width {
                        cell.push('-');
                    }
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
}
