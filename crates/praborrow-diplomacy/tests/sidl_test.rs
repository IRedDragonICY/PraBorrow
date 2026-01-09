use praborrow_diplomacy::Diplomat;

#[test]
fn test_auto_diplomat() {
    #[derive(Diplomat)]
    struct MySovereignData {
        _x: i32,
    }

    // Hash of "MySovereignData" should be stable and non-zero.
    assert_ne!(MySovereignData::TYPE_ID, 0);
    
    // Ensure another struct has a different ID
    #[derive(Diplomat)]
    struct OtherData {}
    
    assert_ne!(MySovereignData::TYPE_ID, OtherData::TYPE_ID);
}
