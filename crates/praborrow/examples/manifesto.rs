use praborrow::core::{Sovereign, CheckProtocol};
use praborrow::defense::Constitution;
use praborrow::logistics::RawResource;

#[derive(Constitution)]
struct StateBudget {
    #[invariant("self.amount > 0")]
    amount: i32,
    #[invariant("self.year >= 2024")]
    year: i32,
}

fn main() {
    println!("--- PraBorrow Manifesto ---");
    praborrow::manifesto();

    // 1. Sovereign Data Test
    println!("\n[1] Testing Sovereign Integrity...");
    let budget = Sovereign::new(StateBudget { amount: 1000, year: 2025 });
    
    // Access safely
    println!("Domestic funds: {}", budget.amount);

    // Annexation
    println!("Annexing funds to foreign jurisdiction...");
    budget.annex().expect("Annexation failed");

    // Attempt retrieval (Should Panic)
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        println!("Attempting to access exiled funds: {}", budget.amount);
    }));

    if result.is_err() {
        println!("VIOLATION INTERCEPTED: Panic caught as expected.");
    } else {
        panic!("TEST FAILED: Sovereignty violation was not punished!");
    }

    // 2. Constitution / Garuda Test
    println!("\n[2] Testing Garuda Constitution...");
    let valid_budget = StateBudget { amount: 500, year: 2025 };
    valid_budget.enforce_law();
    println!("Valid budget passed inspection.");

    let corrupt_budget = StateBudget { amount: -100, year: 2025 };
    let check_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        corrupt_budget.enforce_law();
    }));

    if check_result.is_err() {
        println!("CORRUPTION INTERCEPTED: Negative budget blocked.");
    } else {
        panic!("TEST FAILED: Invariant breach ignored!");
    }

    // 3. Logistics / Hilirisasi Test
    println!("\n[3] Testing Hilirisasi Pipeline...");
    let raw_data = vec![0xCA, 0xFE, 0xBA, 0xBE];
    let resource = RawResource::refine(raw_data);
    
    unsafe {
        println!("Resource refined at {:?}, length: {}", resource.ptr, resource.len);
        let slice = std::slice::from_raw_parts(resource.ptr, resource.len);
        println!("Data content: {:X?}", slice);
    }
    
    println!("\n--- Mission Accomplished ---");
}
