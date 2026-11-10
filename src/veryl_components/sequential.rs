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
pub const VERYL_Y_OUT_ID: &str = "y";

#[derive(Serialize, Deserialize)]
pub struct VerylRegisterComponent {
    pub id: String,
    pub d_in: Input,
}

impl VerylRegisterComponent {
    pub fn new(
        id: &str,
        d_in: Input,
    ) -> Self {
        Self {
            id: id.to_string(),
            d_in,
            
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
                        input: self.d_in.clone(),
                    },
                ],

                OutputType::Sequential,

                vec![
                    VERYL_Y_OUT_ID,
                ],
            ),
        )
    }

    fn clock(&self, _simulator: &mut Simulator) -> Result<(), Condition> {
        Ok(())

    }

    // fn reset(&self) {
    //     if let Some(shared_bridge) = self.bridge.as_ref() {
    //         let mut bridge = shared_bridge.borrow_mut();
    //         bridge.reset().expect("Could not reset Veryl component");
    //     }
    // }

    fn is_sink(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

}

