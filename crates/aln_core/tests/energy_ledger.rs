use aln_core::energy_ledger::*;
use cosmwasm_std::Uint128;

#[test]
fn in_memory_ledger_credit_debit_acl() {
    let mut ledger = InMemoryEnergyLedger::new(1); // system mask bit 1
    let owner = b"owner1";
    // credit
    ledger.credit(owner, EnergyVector { auet: Uint128::new(100), csp: Uint128::new(50), erp: Uint128::new(0) }).unwrap();
    let bal = ledger.balance_of(owner);
    assert_eq!(bal.auet.u128(), 100);
    assert_eq!(bal.csp.u128(), 50);
    // unauthorized debit
    let res = ledger.debit(owner, EnergyVector { auet: Uint128::new(10), csp: Uint128::new(0), erp: Uint128::new(0) }, 0);
    assert!(res.is_err());
    // authorized debit
    let res2 = ledger.debit(owner, EnergyVector { auet: Uint128::new(20), csp: Uint128::new(0), erp: Uint128::new(0) }, 1);
    assert!(res2.is_ok());
    let bal2 = ledger.balance_of(owner);
    assert_eq!(bal2.auet.u128(), 80);
    // underflow
    let res3 = ledger.debit(owner, EnergyVector { auet: Uint128::new(1000), csp: Uint128::new(0), erp: Uint128::new(0) }, 1);
    assert!(res3.is_err());
}
