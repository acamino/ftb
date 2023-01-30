use ftb::TableFormatter;

/// Helper function to format a table
fn format_table(input: &str) -> String {
    let mut formatter = TableFormatter::new();
    formatter.format_table(input)
}

#[test]
fn test_simple_table() {
    let input = include_str!("fixtures/input/simple.txt");
    let expected = include_str!("fixtures/expected/simple.txt");
    let output = format_table(input);
    assert_eq!(output, expected);
}

#[test]
fn test_irregular_table() {
    let input = include_str!("fixtures/input/irregular.txt");
    let expected = include_str!("fixtures/expected/irregular.txt");
    let output = format_table(input);
    assert_eq!(output, expected);
}

#[test]
fn test_complex_table() {
    let input = include_str!("fixtures/input/complex.txt");
    let expected = include_str!("fixtures/expected/complex.txt");
    let output = format_table(input);
    assert_eq!(output, expected);
}

#[test]
fn test_simple_table_structure() {
    let input = include_str!("fixtures/input/simple.txt");
    let output = format_table(input);

    // Verify proper structure
    assert!(output.contains("| h1    | h2    | h3    |"));
    assert!(output.contains("|-"));
    assert!(output.contains("| data1 | data2 | data3 |"));

    // All lines should have the same number of pipes
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 3);
    for line in &lines {
        assert_eq!(line.matches('|').count(), 4);
    }
}

#[test]
fn test_complex_table_handles_missing_cells() {
    let input = include_str!("fixtures/input/complex.txt");
    let output = format_table(input);

    // All lines should have 4 pipes (3 columns)
    for line in output.lines() {
        assert_eq!(line.matches('|').count(), 4, "Line: {}", line);
    }

    // Should handle missing cells by adding empty ones
    assert!(output
        .lines()
        .any(|line| line.contains("d1b") && line.contains("add a cell")));
}
