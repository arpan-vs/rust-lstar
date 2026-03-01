use super::state::State;
use super::transition::Transition;
use super::Automata;
use crate::letter::Letter;
use uuid::Uuid;

/// Lightweight DOT parser assuming the format produced by `Automata::build_dot_code()`
pub fn parse_dot(input: &str) -> Result<Automata, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Input DOT is empty".to_string());
    }

    if !input.starts_with("digraph ") {
        return Err("DOT should start with 'digraph'".to_string());
    }

    let i_start_graph_def = input
        .find('{')
        .ok_or("Missing opening '{' for graph definition")?;

    let automata_name = input[8..i_start_graph_def].trim().trim_matches('"');

    if automata_name.is_empty() {
        return Err("Automata name is empty".to_string());
    }

    let graph_def = input[i_start_graph_def + 1..].trim();
    let graph_entries: Vec<&str> = graph_def.split(';').collect();

    let mut states: Vec<State> = Vec::new();
    for graph_entry in graph_entries {
        if let Err(e) = parse_graph_entry(graph_entry, &mut states) {
            return Err(format!(
                "Error parsing graph entry '{}': {}",
                graph_entry, e
            ));
        }
    }
    Ok(Automata::new(
        State::new("S0".to_string()),
        automata_name.to_string(),
    ))
}

pub fn parse_graph_entry(graph_entry: &str, states: &mut Vec<State>) -> Result<(), String> {
    let graph_entry = graph_entry.trim();
    if graph_entry.is_empty() {
        return Ok(());
    }

    if graph_entry.contains("[shape=") {
        // Parse state definition
        if let Some(start) = graph_entry.find('"') {
            if let Some(end) = graph_entry[start + 1..].find('"') {
                let state_name = &graph_entry[start + 1..start + 1 + end];
                states.push(State::new(state_name.to_string()));
            }
        }
    } else if graph_entry.contains("->") && graph_entry.contains("label=") {
        // Parse transition
        let parts: Vec<&str> = graph_entry.split("->").collect();
        if parts.len() >= 2 {
            let src = extract_quoted_value(parts[0])?;
            let dest = extract_quoted_value(parts[1])?;

            let label = extract_label_value(graph_entry)?;
            let label_parts: Vec<&str> = label.split('/').map(|s| s.trim()).collect();
            let (input, output) = if label_parts.len() >= 2 {
                (label_parts[0].to_string(), label_parts[1].to_string())
            } else {
                (label, "".to_string())
            };

            let t_name =
                extract_url_value(graph_entry).unwrap_or_else(|| Uuid::new_v4().to_string());

            // Note: transitions cannot be added without access to source state
            if let Some(state) = states.iter_mut().find(|s| s.name == src) {
                state.add_transition(Transition::new(
                    t_name,
                    State::new(src),
                    State::new(dest),
                    Letter::new(input),
                    Letter::new(output),
                ));
            } else {
                return Err(format!("Source state '{}' not found for transition", src));
            }
        }
    }

    Ok(())
}

fn extract_quoted_value(s: &str) -> Result<String, String> {
    if let Some(start) = s.find('"') {
        if let Some(end) = s[start + 1..].find('"') {
            return Ok(s[start + 1..start + 1 + end].to_string());
        }
    }
    Err("Quoted value not found".to_string())
}

fn extract_label_value(s: &str) -> Result<String, String> {
    if let Some(label_pos) = s.find("label=\"") {
        let label_start = label_pos + 7; // after label="
        if let Some(label_end_rel) = s[label_start..].find('"') {
            return Ok(s[label_start..label_start + label_end_rel].to_string());
        }
    }
    Err("Label value not found".to_string())
}

fn extract_url_value(s: &str) -> Option<String> {
    if let Some(url_pos) = s.find("URL=") {
        let url_start = url_pos + 4; // after URL=
        if let Some(open_q) = s[url_start..].find('"') {
            let real_start = url_start + open_q + 1;
            if let Some(end_q) = s[real_start..].find('"') {
                return Some(s[real_start..real_start + end_q].to_string());
            }
        }
    }
    None
}

/// Build DOT code representing the provided automata (reverse of `parse_dot`).
pub fn build_dot_code(automata: &Automata) -> String {
    let mut lines: Vec<String> = Vec::new();

    lines.push(format!("digraph \"{}\" {{", automata.name));

    let can_use_flat_transitions = !automata.transitions.is_empty()
        && automata
            .transitions
            .iter()
            .all(|transition| !transition.source_state.name.is_empty());

    if can_use_flat_transitions {
        let mut state_names: Vec<String> = vec![automata.initial_state.name.clone()];
        for transition in &automata.transitions {
            if !state_names.contains(&transition.source_state.name) {
                state_names.push(transition.source_state.name.clone());
            }
            if !state_names.contains(&transition.output_state.name) {
                state_names.push(transition.output_state.name.clone());
            }
        }

        for state_name in &state_names {
            let shape = if state_name == &automata.initial_state.name {
                "doubleoctagon"
            } else {
                "ellipse"
            };
            lines.push(format!(
                "    \"{}\" [shape={}, style=filled, fillcolor=white, URL=\"{}\"];",
                state_name, shape, state_name
            ));
        }

        for transition in &automata.transitions {
            let label = transition.label();
            lines.push(format!(
                "    \"{}\" -> \"{}\" [fontsize=5, label=\"{}\", URL=\"{}\"];",
                transition.source_state, transition.output_state.name, label, transition.name
            ));
        }
    } else {
        // include all states discovered from nested state transitions
        let states = automata.get_states();
        for state in &states {
            let shape = if state.name == automata.initial_state.name {
                "doubleoctagon"
            } else {
                "ellipse"
            };
            lines.push(format!(
                "    \"{}\" [shape={}, style=filled, fillcolor=white, URL=\"{}\"];",
                state.name, shape, state.name
            ));
        }

        for current_state in &states {
            for transition in &current_state.transitions {
                let output_state = &transition.output_state;
                let input = current_state.name.clone();
                let output = output_state.name.clone();
                let label = &transition.label();
                lines.push(format!(
                    "    \"{}\" -> \"{}\" [fontsize=5, label=\"{}\", URL=\"{}\"];",
                    input, output, label, transition.name
                ));
            }
        }
    }

    lines.push("}".to_string());
    lines.join("\n")
}
