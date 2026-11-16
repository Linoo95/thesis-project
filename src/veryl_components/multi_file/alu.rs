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
pub const VERYL_ALU_OUT_ID: &str = "y";

#[derive(Serialize, Deserialize)]
pub struct VerylAluComponent {
    pub id: String,
    pub a: Input,
    pub b: Input,
    pub op: Input,
}

impl VerylAluComponent {
    pub fn new(
        id: &str,
        a: Input,
        b: Input,
        op: Input,
    ) -> Self {
        Self {
            id: id.to_string(),
            a,
            b,
            op,
        }
    }
}

#[typetag::serde]
impl Component for VerylAluComponent {
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
                ],

                OutputType::Combinatorial,

                vec![
                    VERYL_ALU_OUT_ID,
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
