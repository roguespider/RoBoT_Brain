// Test module wiring
#[cfg(test)]
mod tests {
    mod context_assembly_pipeline;
    mod conversation_lifecycle;
    mod data_contract_chain;
    mod pipeline_lifecycle;
}

fn main() {
    // Test suite — tests are in #[cfg(test)] modules
}
