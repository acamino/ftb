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

#[test]
fn test_unicode_emoji_table() {
    let input = include_str!("fixtures/input/unicode_emoji.txt");
    let expected = include_str!("fixtures/expected/unicode_emoji.txt");
    let output = format_table(input);
    assert_eq!(output, expected);
}

#[test]
fn test_unicode_cjk_table() {
    let input = include_str!("fixtures/input/unicode_cjk.txt");
    let expected = include_str!("fixtures/expected/unicode_cjk.txt");
    let output = format_table(input);
    assert_eq!(output, expected);
}

#[test]
fn test_unicode_mixed_table() {
    let input = include_str!("fixtures/input/unicode_mixed.txt");
    let expected = include_str!("fixtures/expected/unicode_mixed.txt");
    let output = format_table(input);
    assert_eq!(output, expected);
}

#[test]
fn test_formatter_reuse_integration() {
    let mut formatter = TableFormatter::new();

    // Format first table
    let input1 = include_str!("fixtures/input/simple.txt");
    let expected1 = include_str!("fixtures/expected/simple.txt");
    let output1 = formatter.format_table(input1);
    assert_eq!(output1, expected1);

    // Reuse formatter for second table
    let input2 = include_str!("fixtures/input/irregular.txt");
    let expected2 = include_str!("fixtures/expected/irregular.txt");
    let output2 = formatter.format_table(input2);
    assert_eq!(output2, expected2);

    // Reuse again for unicode table
    let input3 = include_str!("fixtures/input/unicode_emoji.txt");
    let expected3 = include_str!("fixtures/expected/unicode_emoji.txt");
    let output3 = formatter.format_table(input3);
    assert_eq!(output3, expected3);
}
