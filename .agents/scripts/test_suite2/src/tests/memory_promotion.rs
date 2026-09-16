//! Memory promotion test — T2-134 verification.
use robot_brain::memory::permanent::PermanentMemory;
use robot_brain::memory::retrieve_for_context;
use robot_brain::memory::types::{MemoryItem, MemoryLayer, MemoryType};

#[tokio::test]
async fn test_promote_to_permanent() {
    let memory = PermanentMemory::new(100);
    let item = MemoryItem::new(
        MemoryLayer::Working,
        MemoryType::Knowledge,
        "test content".to_string(),
        "test".to_string(),
    );
    let id = memory
        .promote_to_permanent(item)
        .await
        .expect("promote should succeed");
    assert!(!id.is_empty(), "promoted item should have an id");
}

#[tokio::test]
async fn test_retrieve_for_context() {
    let memory = PermanentMemory::new(100);
    let item = MemoryItem::new(
        MemoryLayer::Permanent,
        MemoryType::Knowledge,
        "retrieval test content".to_string(),
        "test".to_string(),
    );
    memory.store(item).await;
    let results = retrieve_for_context(&memory, "retrieval", 10).await;
    assert!(!results.is_empty(), "should retrieve at least one item");
}
