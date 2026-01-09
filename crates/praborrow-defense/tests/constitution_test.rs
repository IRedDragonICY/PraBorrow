use praborrow_defense::Constitution;

#[test]
fn test_law_enforcement() {
    #[derive(Constitution)]
    struct FiscalData {
        #[invariant("self.value >= 0")]
        value: i32,
    }

    let good = FiscalData { value: 100 };
    good.enforce_law(); // Should not panic

    let bad = FiscalData { value: -5 };
    
    let result = std::panic::catch_unwind(|| {
        bad.enforce_law();
    });
    
    assert!(result.is_err(), "Constitution failed to catch negative value!");
}
