extern crate alloc;
use praborrow::lease::deadlock::WaitForGraph;
use praborrow::prelude::*;
use std::sync::Arc;

/// A Bank Account resource protected by the Constitution.
#[derive(Debug, Constitution)]
pub struct BankAccount {
    /// The account ID.
    pub id: u64,
    /// The balance must never be negative.
    #[invariant("self.balance >= 0")]
    pub balance: i64,
}

impl BankAccount {
    pub fn new(id: u64, balance: i64) -> Self {
        Self { id, balance }
    }

    pub fn deposit(&mut self, amount: i64) {
        self.balance += amount;
    }

    pub fn withdraw(&mut self, amount: i64) -> Result<(), &'static str> {
        if self.balance >= amount {
            self.balance -= amount;
            Ok(())
        } else {
            Err("Insufficient funds")
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏦 Distributed Bank Example: Starting...");

    // 1. Initialize Telemetry (Optional)
    let _telemetry = praborrow::telemetry::TelemetryConfig::builder()
        .service_name("bank-node-1")
        .build();

    // 2. Create a Sovereign Bank Account
    println!("🔐 Creating Sovereign Account #100 with $1000 balance...");
    let account_data = BankAccount::new(100, 1000);
    let mut sovereign_account: Sovereign<BankAccount> = Sovereign::new(account_data); // Invariant checked here

    // 3. Access locally (Domestic Jurisdiction)
    if let Ok(mut account) = sovereign_account.try_get_mut() {
        println!("   Current Balance: ${}", account.balance);
        account.withdraw(100).expect("Withdraw failed");
        println!("   New Balance: ${}", account.balance);

        // Verify Invariants manually (Runtime Check)
        account.enforce_law()?;
        println!("   ✅ Constitution upheld: Balance is non-negative.");
    }

    // 4. Simulate Deadlock Detection (Garuda Dashboard Feature)
    println!("\n🔍 Simulating Transaction Deadlock...");

    let detector = Arc::new(std::sync::Mutex::new(WaitForGraph::new()));

    // Transaction A (Holder 1) locks Account 100, waits for Account 200
    // Transaction B (Holder 2) locks Account 200, waits for Account 100
    // Cycle: 1 -> 200 -> 2 -> 100 -> 1

    {
        let mut graph = detector.lock().unwrap();
        // Holder 1 waits for Resource 200
        graph.add_wait(1, 200);
        // Resource 200 is held by Holder 2
        graph.add_wait(200, 2);
        // Holder 2 waits for Resource 100
        graph.add_wait(2, 100);
        // Resource 100 is held by Holder 1
        graph.add_wait(100, 1);

        if graph.detect_cycle() {
            println!("   ⚠️ DEADLOCK DETECTED! Cycle found in wait-for graph.");
            println!("   Garuda Dashboard would confirm this state via 'verify_deadlock'.");
        } else {
            println!("   No deadlock found.");
        }
    }

    // 5. Demonstrate Formal Verification Stub (Phase 3)
    // In a full implementation, this would call the SMT solver.
    println!("\nzz Verifying Integrity (Formal Proof Stub)...");
    let token = sovereign_account
        .verify_integrity()
        .await
        .expect("Verification failed");
    println!(
        "   Refusing to annex without proof (Safety First): Token received {:?}",
        token
    );

    println!("\n✅ Example completed successfully.");
    Ok(())
}
