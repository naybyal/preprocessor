use regex::Regex;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::toposort;
use std::collections::HashMap;
use std::fs;

fn main() {
    let input_file = "main.c";
    let output_file = "preprocessed_main.c";

    match preprocess_main_c(input_file, output_file) {
        Ok(_) => println!("Preprocessing complete. Output: {}", output_file),
        Err(e) => eprintln!("Error during preprocessing: {}", e),
    }
}

/// Preprocesses a single C file by reordering elements and handling macros.
fn preprocess_main_c(input_file: &str, output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Read the file content
    let original_code = fs::read_to_string(input_file)?;

    // Step 2: Inline #include directives (optional, ignored here as we don't have headers in the example)
    let inlined_code = inline_includes(&original_code)?;

    // Step 3: Reorder code elements
    let reordered_code = reorder_elements(&inlined_code)?;

    // Step 4: Handle macros
    let final_code = handle_macros(&reordered_code)?;

    // Write the preprocessed code to the output file
    fs::write(output_file, final_code)?;

    Ok(())
}

/// Inlines #include directives by replacing them with the content of the referenced files.
fn inline_includes(code: &str) -> Result<String, Box<dyn std::error::Error>> {
    let include_regex = Regex::new(r#"#include\s+"(.+\.h)""#)?;
    let mut inlined_code = String::new();

    for line in code.lines() {
        if let Some(captures) = include_regex.captures(line) {
            let header_file = captures.get(1).unwrap().as_str();
            if let Ok(header_content) = fs::read_to_string(header_file) {
                inlined_code.push_str(&header_content);
            } else {
                eprintln!("Warning: Header file '{}' not found. Skipping include.", header_file);
            }
        } else {
            inlined_code.push_str(line);
            inlined_code.push('\n');
        }
    }

    Ok(inlined_code)
}

/// Reorders code elements (functions, types) in the file based on dependencies.
fn reorder_elements(code: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut graph = DiGraph::<String, ()>::new();
    let mut node_map = HashMap::new();
    let mut functions = Vec::new();

    // Detect function definitions using corrected regex
    let function_regex = Regex::new(r"(\w+\s+\w+\s*\(.*\)\s*\{)")?;
    for (idx, line) in code.lines().enumerate() {
        if let Some(captures) = function_regex.captures(line) {
            let function_name = format!("Function_{}", idx); // Create unique names for functions
            let node = graph.add_node(function_name.clone());
            node_map.insert(function_name.clone(), node);
            functions.push((function_name, idx));
        }
    }

    // Add mock dependencies for simplicity
    for i in 0..functions.len() - 1 {
        let (name_a, _) = &functions[i];
        let (name_b, _) = &functions[i + 1];
        if let (Some(&node_a), Some(&node_b)) = (node_map.get(name_a), node_map.get(name_b)) {
            graph.add_edge(node_a, node_b, ());
        }
    }

    // Perform topological sort
    let sorted_nodes = toposort(&graph, None).map_err(|_| "Cycle detected in dependencies")?;
    let mut reordered_code = String::new();

    for node in sorted_nodes {
        if let Some(function) = functions.iter().find(|(name, _)| *name == graph[node]) {
            reordered_code.push_str(&format!("// Function start: {}\n", function.0));
            reordered_code.push_str(&code.lines().nth(function.1).unwrap());
            reordered_code.push('\n');
        }
    }

    Ok(reordered_code)
}

/// Handles macros by converting them into Rust-compatible constructs.
fn handle_macros(code: &str) -> Result<String, Box<dyn std::error::Error>> {
    let macro_regex = Regex::new(r#"#define\s+(\w+)\s*(.*)"#)?;
    let mut processed_code = String::new();

    for line in code.lines() {
        if let Some(captures) = macro_regex.captures(line) {
            let macro_name = captures.get(1).unwrap().as_str();
            let macro_value = captures.get(2).map_or("", |m| m.as_str());

            // Convert macros into Rust constants or cfg attributes
            if macro_value.is_empty() {
                processed_code.push_str(&format!("#[cfg({})]\n", macro_name));
            } else {
                processed_code.push_str(&format!("const {}: &str = \"{}\";\n", macro_name, macro_value));
            }
        } else {
            processed_code.push_str(line);
            processed_code.push('\n');
        }
    }

    Ok(processed_code)
}
