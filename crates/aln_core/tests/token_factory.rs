use aln_core::{TokenFactory, TokenClassParams, TokenClassType};
use crate::energy_ledger::InMemoryEnergyLedger;

#[test]
fn create_energy_token_class_requires_aln_fee() {
    let params = TokenClassParams { name: "AU.ET".to_string(), symbol: "AUET".to_string(), class_type: TokenClassType::Aln20Energy, template_id: None };
    let res = TokenFactory::create_token_class(&params, "creator", 9);
    assert!(res.is_err());
    let ok = TokenFactory::create_token_class(&params, "creator", 10).unwrap();
    assert_eq!(ok.params.class_type, TokenClassType::Aln20Energy);
}

#[test]
fn token_class_create_requires_role() {
    let params = TokenClassParams { name: "Foo".to_string(), symbol: "FOO".to_string(), class_type: TokenClassType::Other, template_id: None };
    // 'creator' did not include 'operator' or 'builder' in identity mapping stub
    let res = TokenFactory::create_token_class(&params, "user_did", 1);
    assert!(res.is_err());
    // Creator with role in DID mapping should pass
    let res2 = TokenFactory::create_token_class(&params, "operator_did", 1).unwrap();
    assert_eq!(res2.params.class_type, TokenClassType::Other);
}

#[test]
fn token_factory_mint_to_ledger_credits() {
    let mut ledger = InMemoryEnergyLedger::new(1);
    let owner = b"ownerX";
    // Mint 100 units as energy
    TokenFactory::mint_to_ledger(&mut ledger, owner, 100).unwrap();
    let bal = ledger.balance_of(owner);
    assert_eq!(bal.auet.u128(), 100);
}
