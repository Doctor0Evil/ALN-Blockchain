pub trait EthicsBoard {
    fn ethics_board_id(&self) -> String;
}

pub trait RegulatorQuorum {
    fn regulator_quorum_id(&self) -> String;
}

pub trait PatientConsent {
    fn consent_token(&self) -> String;
}
