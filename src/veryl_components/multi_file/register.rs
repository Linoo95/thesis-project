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


pub const VERYL_D_IN_ID: &str = "d";
pub const VERYL_ENABLE_IN_ID: &str = "enable";

pub const VERYL_Q_OUT_ID: &str = "reg_out";

#[derive(Serialize, Deserialize)]
pub struct VerylRegisterComponent {
    pub id: String,
    pub enable: Input,
    pub d: Input,
}

impl VerylRegisterComponent {
    pub fn new(
        id: &str,
        enable: Input,
        d: Input,
    ) -> Self {
        Self {
            id: id.to_string(),
            enable,
            d,
        }
    }
}

#[typetag::serde]
impl Component for VerylRegisterComponent {
    fn get_id_ports(&self) -> (Id, Ports) {
        (
            self.id.clone(),
            Ports::new(
                vec![
                    &InputPort {
                        port_id: VERYL_D_IN_ID.to_string(),
                        input: self.d.clone(),
                    },
                    &InputPort {
                        port_id: VERYL_ENABLE_IN_ID.to_string(),
                        input: self.enable.clone(),
                    },
                ],

                OutputType::Sequential,

                vec![
                    VERYL_Q_OUT_ID,
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