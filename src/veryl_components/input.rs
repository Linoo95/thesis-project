use std::{any::Any};

use serde::{Deserialize, Serialize};

use syncrim::{
    common::{
        Component,
        Condition,
        Id,
        OutputType,
        Ports,
        Simulator,
    }
};



pub const VERYL_INPUT_OUT_ID: &str = "a";

#[derive(Serialize, Deserialize)]
pub struct VerylInputComponent {
    pub id: String,
}

impl VerylInputComponent {
    pub fn new(
        id: &str,
    
    ) -> Self {
        Self {
            id: id.to_string(),
            
        }
    }
}
#[typetag::serde]
impl Component for VerylInputComponent {
    fn get_id_ports(&self) -> (Id, Ports) {
        (
            self.id.clone(),
            Ports::new(
                vec![],
                OutputType::Combinatorial,
                vec![
                    VERYL_INPUT_OUT_ID
                ],
            )
        )
    }

    fn clock(&self, _simulator: &mut Simulator) -> Result<(), Condition> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

}