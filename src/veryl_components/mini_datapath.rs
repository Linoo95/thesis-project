use std::{any::Any};

use serde::{Deserialize, Serialize};

use syncrim::common::{
        Component, Condition, Id, Input, OutputType, Ports, Simulator,
    };



pub const RESULT_OUT_ID: &str = "result";

#[derive(Serialize, Deserialize)]
pub struct MiniDatapathComponent {
    pub id: String,
    pub a: Input,
    pub b: Input,
    pub op: Input,
    pub enable: Input,
}

impl MiniDatapathComponent {
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
impl Component for MiniDatapathComponent {
    fn get_id_ports(&self) -> (Id, Ports) {
        (
            self.id.clone(),
            Ports::new(
                vec![],
                OutputType::Sequential,
                vec![
                    RESULT_OUT_ID
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