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
pub const VERYL_Y_OUT_ID: &str = "y";

#[derive(Serialize, Deserialize)]
pub struct VerylCombinationalComponent {
    pub id: String,
    pub a_in: Input,
}

impl VerylCombinationalComponent {
    pub fn new(
        id: &str,
        a_in: Input,     
        
    ) -> Self {
        Self {
            id: id.to_string(),
            a_in,
        }
    }
}
#[typetag::serde]
impl Component for VerylCombinationalComponent {
    fn get_id_ports(&self) -> (Id, Ports) {
        (
            self.id.clone(),
            Ports::new(
                vec![
                    &InputPort{
                        port_id: VERYL_A_IN_ID.to_string(),
                        input: self.a_in.clone(),
                    },
                ], 
                OutputType::Combinatorial, 
                vec![
                    VERYL_Y_OUT_ID
                ],
            )
        )
    }

    fn clock(&self, _simulator: &mut Simulator) -> Result<(), Condition> {
        Ok(())
    }
    

    fn as_any(&self) ->  &dyn Any {
        self
    }
}

