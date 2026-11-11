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

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, path::PathBuf, rc::Rc};

    use syncrim::{
        common::{
            ComponentStore,
            Input,
            Signal,
            SignalFmt::Unsigned,
            SignalSize,
            
            Simulator,
        }, components::{
            ADD_OUT_ID, CONSTANT_OUT_ID, Constant,  Add,
        },
    };

    use crate::{
        PortType, VERYL_A_OUT_ID, VerylInputComponent,  combinational::{
            VERYL_Y_OUT_ID, VerylCombinationalComponent, 
        }, hybrid_simulator::{HybridSimulator,  VerylToSyncRimMapping}, veryl_bridge::VerylBridge, veryl_loader::load_veryl_simulator,
    };

    #[test]
    fn native_syncrim_comb_after_veryl(){
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join("test_bridge.veryl");

        let veryl_design = load_veryl_simulator(file.to_str().unwrap(), "TestBridge").unwrap();

        let shared_bridge = Rc::new(
            RefCell::new(
                VerylBridge::new(
                    veryl_design.simulator,
                    veryl_design.design_info,
                ).unwrap()
            )
        );
        let veryl_input = Rc::new(
            VerylInputComponent::new(
                "veryl_input"
            )
        );

        let veryl_comb = Rc::new(         
            VerylCombinationalComponent::new(
                "veryl_comb", 
                Input::new(
                    "veryl_input", 
                    VERYL_A_OUT_ID
                ),
            )          
        );
        let mut signal: Signal = 4.into();
        signal.set_fmt(Unsigned(SignalSize::_8));

        let constant = Rc::new(
            Constant::new(
                "constant_id",
                (1.0, 1.0), 
                signal
            )
        );

        let add_syncrim = Rc::new(
            Add::new(
                "add_syncrim", 
                (0.0, 0.0), 
                Input::new(
                    "constant_id", 
                    CONSTANT_OUT_ID
                ), 
                Input::new(
                    "veryl_comb",
                    VERYL_Y_OUT_ID,
                )
            )
        );
        let store = ComponentStore {
            store: vec![
                veryl_input,
                veryl_comb,
                constant,
                add_syncrim,
            ],
        };

        let simulator = Simulator::new(store).unwrap();

        let veryl_to_syncrim_mappings = vec![
            VerylToSyncRimMapping {
                veryl_port: "a".into(),
                syncrim_component: "veryl_input".into(),
                syncrim_output: VERYL_A_OUT_ID.into(),
                destination_type: PortType { width: 8, signed: false }
            },
            VerylToSyncRimMapping {
                veryl_port: "y".into(),
                syncrim_component: "veryl_comb".into(),
                syncrim_output: VERYL_Y_OUT_ID.into(),
                destination_type: PortType { width: 8, signed: false }
            },
        ];

        let syncrim_to_veryl_mappings = vec![];

        let mut hybrid = HybridSimulator::new(simulator, shared_bridge.clone(), veryl_to_syncrim_mappings, syncrim_to_veryl_mappings);
        
        hybrid.syncrim.set_out_fmt("add_syncrim", ADD_OUT_ID, Unsigned(SignalSize::_8));
        println!("state: {:?}", hybrid.syncrim.get_state());
        println!("conditions: {:?}", hybrid.syncrim.get_component_condition());
        println!("cycle: {}", hybrid.syncrim.cycle);
    
        let veryl_comb = hybrid.syncrim.get_input_signal(&Input::new("veryl_comb", VERYL_Y_OUT_ID));
        let syncrim_add = hybrid.syncrim.get_input_signal(&Input::new("add_syncrim", ADD_OUT_ID));
        
        println!("Before SyncRim propogation:");
        println!("Syncrim Add: {:?}", syncrim_add);
        println!("Veryl Comb: {:?}", veryl_comb);

        let a: Signal = 5.into();
        
        let port_type = PortType {
            width: 8,
            signed: false,
        };
        hybrid.set_veryl_input("a", a.get_value(), port_type).unwrap();
        
        let syncrim_add = hybrid.syncrim.get_input_signal(&Input::new("add_syncrim", ADD_OUT_ID));
        let veryl_comb = hybrid.syncrim.get_input_signal(&Input::new("veryl_comb", VERYL_Y_OUT_ID));
        
        println!("After Veryl sync: ");
        println!("Veryl comb: {:?}", veryl_comb);
        println!("Add: {:?}", syncrim_add);

        hybrid.syncrim.clock();
        
        let syncrim_add = hybrid.syncrim.get_input_signal(&Input::new("add_syncrim", ADD_OUT_ID));
        println!("After SyncRim propagation: {:?}", syncrim_add);




    }
    // #[test]
    // fn only_veryl_components(){
    //     let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join("test_bridge_comb_seq.veryl");

    //     let veryl_simulator = load_veryl_simulator(file.to_str().unwrap(), "TestCombSeq");

    //     let shared_bridge = Rc::new(
    //         RefCell::new(
    //             VerylBridge::with_clock_reset(
    //                 veryl_simulator,
    //             "clk",
    //             "rst",
    //             ).unwrap()
    //         )
    //     );

    //     let veryl_input = Rc::new(
    //         VerylInputComponent::new(
    //             "veryl_input", 
    //             shared_bridge.clone()
    //         )
    //     );

    //     let veryl_comb = Rc::new(         
    //         VerylCombinationalComponent::new(
    //             "veryl_comb", 
    //             Input::new(
    //                 "veryl_input", 
    //                 VERYL_A_OUT_ID
    //             ), 
    //             shared_bridge.clone(),
    //             "a",
    //             "y",
    //         )          
    //     );

    //     let veryl_seq = Rc::new(           
    //         VerylRegisterComponent::new(
    //             "veryl_seq", 
    //             Input::new(
    //                 "veryl_comb", 
    //                 VERYL_Y_OUT_ID
    //             ), 
    //             shared_bridge.clone()
    //         )       
    //     );

    //     let store = ComponentStore {
    //         store: vec![
    //             veryl_input,
    //             veryl_comb,
    //             veryl_seq,
    //         ],
    //     };

    //     {
    //         let mut bridge = shared_bridge.borrow_mut();

    //         let mut a: Signal = 10.into();
    //         a.set_fmt(Unsigned(SignalSize::_8));

    //         bridge.set_input("a", a).unwrap();

    //     }
    //     let mut simulator = Simulator::new(store).unwrap();

    //     let mappings = vec![
    //         VerylToSyncRimMapping {
    //             veryl_port: "comb_out".into(),
    //             syncrim_component: "veryl_comb".into(),
    //             syncrim_output: VERYL_Y_OUT_ID.into()
    //         },
    //         VerylToSyncRimMapping {
    //             veryl_port: "y".into(),
    //             syncrim_component: "veryl_seq".into(),
    //             syncrim_output: VERYL_Y_OUT_ID.into()
    //         },
    //     ];

    //     let mut hybrid = HybridCoordinator::new(simulator, shared_bridge.clone(), mappings).unwrap();


    //     let veryl_comb = hybrid.syncrim.get_input_signal(&Input::new("veryl_comb", VERYL_Y_OUT_ID));
    //     let veryl_seq = hybrid.syncrim.get_input_signal(&Input::new("veryl_seq", VERYL_Y_OUT_ID));
        
    //     assert_eq!(veryl_comb.get_value(), SignalValue::Data(11));
    //     assert_eq!(veryl_seq.get_value(), SignalValue::Data(0));
        

    //     let mut a: Signal = 5.into();
    //     a.set_fmt(Unsigned(SignalSize::_8));

    //     hybrid.set_veryl_input("a", a).unwrap();
    
    //     let veryl_comb = hybrid.syncrim.get_input_signal(&Input::new("veryl_comb", VERYL_Y_OUT_ID));
    //     let veryl_seq = hybrid.syncrim.get_input_signal(&Input::new("veryl_seq", VERYL_Y_OUT_ID));
    //     assert_eq!(veryl_comb.get_value(), SignalValue::Data(6));
    //     assert_eq!(veryl_seq.get_value(), SignalValue::Data(0));
        
    //     hybrid.clock_veryl().unwrap();
        
    //     let veryl_comb = hybrid.syncrim.get_input_signal(&Input::new("veryl_comb", VERYL_Y_OUT_ID));
    //     let veryl_seq = hybrid.syncrim.get_input_signal(&Input::new("veryl_seq", VERYL_Y_OUT_ID));
    //     assert_eq!(veryl_comb.get_value(), SignalValue::Data(6));
    //     assert_eq!(veryl_seq.get_value(), SignalValue::Data(6));


    // }

    // #[test]
    // fn veryl_comb_veryl_sync(){
    //     let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join("test_bridge_comb_seq.veryl");

    //     let veryl_simulator = load_veryl_simulator(file.to_str().unwrap(), "TestCombSeq");

    //     let shared_bridge = Rc::new(
    //         RefCell::new(
    //             VerylBridge::with_clock_reset(
    //                 veryl_simulator,
    //             "clk",
    //             "rst",
    //             ).unwrap()
    //         )
    //     );
    //     let mut signal: Signal = 10.into();
    //     signal.set_fmt(Unsigned(SignalSize::_8));

    //     let constant = Rc::new(
    //         Constant::new(
    //             "constant_id",
    //             (1.0, 1.0), 
    //             signal
    //         )
    //     );

    //     let veryl_comb = Rc::new(         
    //         VerylCombinationalComponent::new(
    //             "veryl_comb", 
    //             Input::new(
    //                 "constant_id", 
    //                 CONSTANT_OUT_ID
    //             ), 
    //             shared_bridge.clone(),
    //             "a",
    //             "y",
    //         )          
    //     );

    //     let veryl_seq = Rc::new(           
    //         VerylRegisterComponent::new(
    //             "veryl_seq", 
    //             Input::new(
    //                 "veryl_comb", 
    //                 VERYL_Y_OUT_ID
    //             ), 
    //             shared_bridge.clone()
    //         )       
    //     );

    //     let store = ComponentStore {
    //         store: vec![
    //             constant,
    //             veryl_comb,
    //             veryl_seq,
    //         ],
    //     };

    //     let mut simulator = Simulator::new(store).unwrap();

    //     simulator.set_out_fmt(
    //         "constant_id", 
    //         CONSTANT_OUT_ID, 
    //         Unsigned(SignalSize::_8)
    //     );
    //     println!("cycle: {}", simulator.cycle);
    //     let veryl_comb_out = simulator.get_input_signal(&Input::new("veryl_comb", VERYL_Y_OUT_ID));
    //     let veryl_seq_out = simulator.get_input_signal(&Input::new("veryl_seq", VERYL_Y_OUT_ID));

    //     println!("Veryl comb out: {:?}\nVeryl seq out: {:?}", veryl_comb_out, veryl_seq_out);

    //     simulator.clock();

    //     println!("cycle: {}", simulator.cycle);
    //     let veryl_comb_out = simulator.get_input_signal(&Input::new("veryl_comb", VERYL_Y_OUT_ID));
    //     let veryl_seq_out = simulator.get_input_signal(&Input::new("veryl_seq", VERYL_Y_OUT_ID));

    //     println!("Veryl comb out: {:?}\nVeryl seq out: {:?}", veryl_comb_out, veryl_seq_out);

    //     simulator.clock();

    //     println!("cycle: {}", simulator.cycle);
    //     let veryl_comb_out = simulator.get_input_signal(&Input::new("veryl_comb", VERYL_Y_OUT_ID));
    //     let veryl_seq_out = simulator.get_input_signal(&Input::new("veryl_seq", VERYL_Y_OUT_ID));

    //     println!("Veryl comb out: {:?}\nVeryl seq out: {:?}", veryl_comb_out, veryl_seq_out);


    // }
    // #[test]
    // fn veryl_comb_sync_comb(){
    //     let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join("test_bridge.veryl");

    //     let veryl_simulator = load_veryl_simulator(file.to_str().unwrap(), "TestBridge");

    //     let shared_bridge = Rc::new(
    //         RefCell::new(
    //             VerylBridge::new(veryl_simulator)
    //         )
    //     );
    //     let mut input: Signal = 10.into();
    //     input.set_fmt(Unsigned(SignalSize::_8));

    //     let constant = Rc::new(
    //         Constant::new(
    //             "constant",
    //             (0.0, 0.0),
    //             input
    //         )
    //     );

    //     let veryl_comb = Rc::new(
    //         VerylCombinationalComponent::new(
    //             "veryl_comb",
    //             Input::new(
    //                 "constant", 
    //                 CONSTANT_OUT_ID
    //             ), 
    //             shared_bridge.clone(),
    //             "a",
    //             "y",
    //         )
    //     );

    //     let store = ComponentStore { 
    //         store: vec![
    //             constant,
    //             veryl_comb
    //         ] 
    //     };

    //     let mut simulator = Simulator::new(store).unwrap();

        
    //     simulator.set_out_fmt(
    //         "constant", 
    //         CONSTANT_OUT_ID, 
    //         Unsigned(SignalSize::_8)
    //     );
    //     let mut veryl_out = simulator.get_input_signal(&Input::new("veryl_comb", VERYL_Y_OUT_ID));
    //     // y = 0 + 1;
    //     assert_eq!(veryl_out.get_value(), SignalValue::Data(1));

    //     simulator.clock();
    //     // y = 10 + 1;
    //     veryl_out = simulator.get_input_signal(&Input::new("veryl_comb", VERYL_Y_OUT_ID));
    //     assert_eq!(veryl_out.get_value(), SignalValue::Data(11));
        


    // }
    // #[test]
    // fn veryl_combinational_syncrim_sequential(){

    //     let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join("test_bridge.veryl");

    //     let veryl_simulator = load_veryl_simulator(file.to_str().unwrap(), "TestBridge");

    //     let bridge = Rc::new(
    //         RefCell::new(
    //             VerylBridge::new(veryl_simulator)
    //         )
    //     );


    //     let mut input_signal: Signal = 10.into();
    //     input_signal.set_fmt(Unsigned(SignalSize::_8));

    //     let constant = Rc::new(
    //         Constant::new(
    //             "input",
    //             (0.0, 0.0),
    //             input_signal,
    //         )
    //     );

    //     let veryl_component = Rc::new(
    //         VerylCombinationalComponent::new(
    //             "veryl_combinational", 
    //             Input::new(
    //                 "input", 
    //                 CONSTANT_OUT_ID
    //             ), 
    //             bridge,
    //             "a",
    //             "y",
    //         )
    //     );

    //     let register_component = Rc::new(
    //         Register::new(
    //             "register_component",
    //             (0.0, 0.0),
    //             Input::new("veryl_combinational", VERYL_Y_OUT_ID),
    
    //         ),
    //     );


    //     let store = ComponentStore {
    //         store: vec![
    //             constant,
    //             veryl_component,
    //             register_component,
    //         ]
    //     };

    //     let mut simulator = Simulator::new(store).unwrap();

    //     simulator.set_out_fmt(
    //         "input",
    //         CONSTANT_OUT_ID,
    //         Unsigned(SignalSize::_8),
    //     );

    //     simulator.clock();

    //     let combinational_output = simulator.get_input_signal(
    //         &Input::new (
    //             "veryl_combinational", 
    //             VERYL_Y_OUT_ID ,
    //         )
    //     );
    //     let mut register_output = simulator.get_input_signal(
    //         &Input::new(
    //             "register_component",
    //             REGISTER_OUT_ID,

    //         )
    //     );
    //     assert_eq!(register_output.get_value(), SignalValue::Data(1));
    //     assert_eq!(combinational_output.get_value(), SignalValue::Data(11));

    //     simulator.clock();

    //     register_output = simulator.get_input_signal(
    //         &Input::new(
    //             "register_component",
    //             REGISTER_OUT_ID,

    //         )
    //     );

    //     assert_eq!(register_output.get_value(), SignalValue::Data(11));



    // }

    // #[test]
    // fn syncrim_comb_veryl_comb_syncrim_comb_propergates_in_one_cycle(){
    //     let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join("test_bridge.veryl");

    //     let veryl_simulator = load_veryl_simulator(file.to_str().unwrap(), "TestBridge");

    //     let bridge = Rc::new(
    //         RefCell::new(
    //             VerylBridge::new(veryl_simulator)
    //         )
    //     );

    //     let mut input_10: Signal = 10.into();
    //     input_10.set_fmt(Unsigned(SignalSize::_8));

    //     let mut input_5: Signal = 5.into();
    //     input_5.set_fmt(Unsigned(SignalSize::_8));

    //     let mut input_4: Signal = 4.into();
    //     input_4.set_fmt(Unsigned(SignalSize::_8));

    //     let constant_10 = Rc::new(
    //         Constant::new(
    //             "input_10",
    //             (0.0, 0.0),
    //             input_10
    //         )
    //     );

    //     let constant_5 = Rc::new(
    //         Constant::new(
    //             "input_5",
    //             (1.0, 1.0),
    //             input_5)
    //     );

    //     let constant_4 = Rc::new(
    //         Constant::new(
    //             "input_4",
    //             (1.0, 1.0),
    //             input_4)
    //     );

    //     let add_component_1 = Rc::new(
    //         Add::new(
    //             "add_component_1", 
    //             (2.0, 2.0), 
    //             Input::new(
    //                 "input_10", 
    //                 CONSTANT_OUT_ID
    //             ), 
    //             Input::new(
    //                 "input_5", 
    //                 CONSTANT_OUT_ID,
    //             )
    //         )
    //     );
    //     let veryl_component = Rc::new(
    //         VerylCombinationalComponent::new(
    //             "veryl_component", 
    //             Input::new(
    //                 "add_component_1",
    //                 ADD_OUT_ID,
    //             ), 
    //             bridge,
    //             "a",
    //             "y",
    //         )
    //     );

    //     let add_component_2 = Rc::new(
    //         Add::new(
    //             "add_component_2", 
    //             (2.0, 2.0), 
    //             Input::new(
    //                 "veryl_component", 
    //                 VERYL_Y_OUT_ID,
    //             ), 
    //             Input::new(
    //                 "input_4", 
    //                 CONSTANT_OUT_ID,
    //             )
    //         )
    //     );



    //     let store = ComponentStore {
    //         store: vec![
    //             constant_10,
    //             constant_4,
    //             constant_5,
    //             add_component_1,
    //             add_component_2,
    //             veryl_component,
    //         ]
    //     };

    //     let mut simulator = Simulator::new(store).unwrap();

    //     simulator.set_out_fmt(
    //         "add_component_1", 
    //         ADD_OUT_ID, 
    //         Unsigned(SignalSize::_8)
    //     );

    //     simulator.set_out_fmt(
    //         "add_component_2", 
    //         ADD_OUT_ID, 
    //         Unsigned(SignalSize::_8)
    //     );

    //     simulator.set_out_fmt(
    //         "input_10", 
    //         CONSTANT_OUT_ID, 
    //         Unsigned(SignalSize::_8)
    //     );

    //     simulator.set_out_fmt(
    //         "input_5", 
    //         CONSTANT_OUT_ID,
    //         Unsigned(SignalSize::_8)
    //     );

    //     simulator.set_out_fmt(
    //         "input_4", 
    //         CONSTANT_OUT_ID,
    //         Unsigned(SignalSize::_8)
    //     );

    //     simulator.set_out_fmt(
    //         "veryl_component", 
    //         VERYL_Y_OUT_ID,
    //         Unsigned(SignalSize::_8)
    //     );

    //     let mut add_1_out = simulator.get_input_signal(
    //         &Input::new(
    //             "add_component_1", ADD_OUT_ID
    //         )
    //     );

    //     let mut add_2_out = simulator.get_input_signal(
    //         &Input::new(
    //             "add_component_2", ADD_OUT_ID
    //         )
    //     );

    //     let mut veryl_out = simulator.get_input_signal(
    //         &Input::new(
    //             "veryl_component", VERYL_Y_OUT_ID
    //         )
    //     );

    //     assert_eq!(add_1_out.get_value(), SignalValue::Data(15));
    //     assert_eq!(veryl_out.get_value(), SignalValue::Data(1));
    //     assert_eq!(add_2_out.get_value(), SignalValue::Data(5));
    //     simulator.clock();


    //     add_1_out = simulator.get_input_signal(
    //         &Input::new(
    //             "add_component_1", ADD_OUT_ID
    //         )
    //     );

    //     add_2_out = simulator.get_input_signal(
    //         &Input::new(
    //             "add_component_2", ADD_OUT_ID
    //         )
    //     );

    //     veryl_out = simulator.get_input_signal(
    //         &Input::new(
    //             "veryl_component", VERYL_Y_OUT_ID
    //         )
    //     );

    //     assert_eq!(add_1_out.get_value(), SignalValue::Data(15));
    //     assert_eq!(veryl_out.get_value(), SignalValue::Data(16));
    //     assert_eq!(add_2_out.get_value(), SignalValue::Data(20));

    // }
}
