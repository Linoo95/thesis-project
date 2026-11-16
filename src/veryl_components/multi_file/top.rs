use std::{
    any::Any, 
    
};
use serde::{Deserialize, Serialize};
use syncrim::{
    common::{
        Component,
        Condition,
        Id,
        Input,
        InputPort,
        OutputType,
        Ports,
        Simulator
    },
};


pub const VERYL_A_IN_ID: &str = "a";
pub const VERYL_B_IN_ID: &str = "b";
pub const VERYL_OP_IN_ID: &str = "op";
pub const VERYL_ENABLE_IN_ID: &str = "enable";
pub const VERYL_RESULT_OUT_ID: &str = "result";

#[derive(Serialize, Deserialize)]
pub struct VerylTopComponent {
    pub id: String,
    pub a: Input,
    pub b: Input,
    pub op: Input,
    pub enable: Input,
}

impl VerylTopComponent {
    pub fn new(
        id: &str,
        a: Input,
        b: Input,
        op: Input,
        enable: Input,
    ) -> Self {
        Self {
            id: id.to_string(),
            a,
            b,
            op,
            enable,
        }
    }
}

#[typetag::serde]
impl Component for VerylTopComponent {
    fn get_id_ports(&self) -> (Id, Ports) {
        (
            self.id.clone(),
            Ports::new(
                vec![
                    &InputPort {
                        port_id: VERYL_A_IN_ID.to_string(),
                        input: self.a.clone(),
                    },
                    &InputPort {
                        port_id: VERYL_B_IN_ID.to_string(),
                        input: self.b.clone(),
                    },
                    &InputPort {
                        port_id: VERYL_OP_IN_ID.to_string(),
                        input: self.op.clone(),
                    },
                    &InputPort {
                        port_id: VERYL_ENABLE_IN_ID.to_string(),
                        input: self.enable.clone(),
                    },
                ],

                OutputType::Combinatorial,

                vec![
                    VERYL_RESULT_OUT_ID,
                ],
            ),
        )
    }

    fn clock(&self, _simulator: &mut Simulator) -> Result<(), Condition> {
        Ok(())

    }

    fn is_sink(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

}
