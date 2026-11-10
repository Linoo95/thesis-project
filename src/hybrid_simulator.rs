

use core::sync;

use syncrim::{
    common::{
        Input,
        OutputType,
        Simulator
    }, signal::SignalValue, simulator,
};


use crate::{PortDirection, PortType, SharedVerylBridge, VerylBridge, convert_veryl_to_syncrim};

#[derive(Clone)]
pub struct VerylToSyncRimMapping {
    pub veryl_port: String,
    pub syncrim_component: String,
    pub syncrim_output: String,

    pub destination_type: PortType,
}

#[derive(Clone)]
pub struct SyncRimToVerylMapping {
    pub syncrim_component: String,
    pub syncrim_output: String,

    pub source_type: PortType,

    pub veryl_port: String,
}

#[derive(Clone)]
pub struct PortConnection {
    pub veryl_port: String,
    pub syncrim_component: String,
    pub syncrim_port: String,
}
const MAX_SETTLE_ITERATIONS: usize = 100;
#[derive(Clone)]
pub struct HybridSimulator {
    pub syncrim: Simulator,
    pub veryl: SharedVerylBridge,

    pub cycle: usize,

    veryl_to_syncrim: Vec<VerylToSyncRimMapping>,
    syncrim_to_veryl: Vec<SyncRimToVerylMapping>,
}

impl HybridSimulator {
    pub fn new(syncrim: Simulator, 
        veryl: SharedVerylBridge,
        veryl_to_syncrim: Vec<VerylToSyncRimMapping>,
        syncrim_to_veryl: Vec<SyncRimToVerylMapping>,
    ) -> Self {
        Self {
            syncrim,
            veryl,
            cycle: 0,
            veryl_to_syncrim,
            syncrim_to_veryl,
        }
    }

    pub fn initialize_state(&mut self) -> Result<(), String> {
        self.settle_combinational()
    }

    pub fn clock(&mut self) -> Result<(), String> {
        // PRE CLOCK
        // all comb values must be stable
        self.settle_combinational()?;

        // CLOCK EDGE:
        // no sync between these calls
        self.clock_veryl_sequential()?;
        self.clock_syncrim_sequential()?;

        // POST CLOCK
        // publish new register caues and settle resulting logic
        self.settle_combinational()?;

        self.cycle += 1;
        Ok(())
    }

    fn clock_veryl_sequential(&mut self) -> Result<(), String> { 
        let mut bridge = self.veryl.borrow_mut();

        if bridge.has_clock() {
            bridge.clock()?;
        }

        Ok(())
    }

    fn clock_syncrim_sequential(&mut self) -> Result<(), String> {
        for component in self.syncrim.ordered_components.clone() {
            let (_, ports) = component.get_id_ports();

            if ports.out_type == OutputType::Sequential {
                component
                    .clock(&mut self.syncrim)
                    .map_err(|condition| {
                        format!("Syncrim sequential component {} failed: {:?}", component.get_id_ports().0, condition)
                    })?;

            }
        }
        Ok(())
    }

    fn settle_syncrim_combinational(&mut self) -> Result<(), String>{
        for component in self.syncrim.ordered_components.clone() {
            let (_, ports) = component.get_id_ports();

            if ports.out_type == OutputType::Combinatorial {
                component
                    .clock(&mut self.syncrim)
                    .map_err(|condition| {
                        format!("Syncrim combinational component {} failed: {:?}", component.get_id_ports().0, condition)
                    })?;

            }
        }
        Ok(())
    }
    pub fn settle_combinational(&mut self,) -> Result<(), String>{
        let mut iterations = 0;
        for _ in 0..MAX_SETTLE_ITERATIONS {

            let before = self.read_boundary_state()?;
            self.sync_syncrim_to_veryl()?;

            self.sync_veryl_to_syncrim()?;

            self.settle_syncrim_combinational()?;

            let after = self.read_boundary_state()?;
            iterations += 1;
            if before == after {
                println!("Iterations: {}", iterations);
                return Ok(());
            }
        }
        Err(format!("Hybrid combinational logic failed after {} iterations", MAX_SETTLE_ITERATIONS))
    }

    fn sync_syncrim_to_veryl(&mut self) -> Result<(), String> {
        for mapping in &self.syncrim_to_veryl {
            let signal = self.syncrim.get_input_signal(
                &Input::new(
                    &mapping.syncrim_component, 
                    &mapping.syncrim_output,
                )
            );
            let mut bridge = self.veryl.borrow_mut();
            bridge.set_input(&mapping.veryl_port, signal.get_value(), mapping.source_type)?;
        }
        Ok(())
    }

    fn sync_veryl_to_syncrim(&mut self) -> Result<(), String> {

        let mut bridge = self.veryl.borrow_mut();

        for mapping in &self.veryl_to_syncrim {
            let source_type = bridge.port_type(&mapping.veryl_port)?;

            let value = bridge.read_value(&mapping.veryl_port)?;

            let signal_value = convert_veryl_to_syncrim(value, source_type, mapping.destination_type)?;

            self.syncrim.set_out_value(
                &mapping.syncrim_component, 
                &mapping.syncrim_output, 
                signal_value,
            );
        }
        Ok(())
    }

    fn read_boundary_state(&mut self) -> Result<Vec<SignalValue>, String> {
        let mut input_values: Vec<SignalValue> = vec![];

        for mapping in &self.syncrim_to_veryl {
            let signal = self.syncrim.get_input_signal(
                &Input::new(
                    &mapping.syncrim_component, 
                    &mapping.syncrim_output,
                )
            );
            input_values.push(signal.get_value());
        }
        let mut bridge = self.veryl.borrow_mut();

        for mapping in &self.veryl_to_syncrim {
            let source_type = bridge.port_type(&mapping.veryl_port)?;
            
            let value = bridge.read_value(&mapping.veryl_port)?;

            let signal_value = convert_veryl_to_syncrim(value, source_type, mapping.destination_type)?;

            input_values.push(signal_value);
        }
        Ok(input_values)
    }

    pub fn set_veryl_input(
        &mut self,
        port: &str,
        value: SignalValue,
        source_type: PortType,
    ) -> Result<(), String> {
        self.veryl.borrow_mut().set_input(port, value, source_type)
    }

}

pub fn create_mappings(
    bridge: &VerylBridge,
    syncrim: &Simulator,
) -> Result<(Vec<VerylToSyncRimMapping>, Vec<SyncRimToVerylMapping>), String> {
    let mut veryl_to_syncrim = Vec::new();
    let mut syncrim_to_veryl = Vec::new();

    for port in &bridge.design_info.ports {
        match port.direction {
            PortDirection::Input => {
                // find syncrim source
            }
            PortDirection::Output => {
                // find syncrim destination
            }
            _ => {

            }
            
        }
    }
    Ok((veryl_to_syncrim, syncrim_to_veryl))
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, path::PathBuf, rc::Rc};

    use syncrim::{
        common::{
            ComponentStore,
            Input,
            Signal,
            SignalValue,
            Simulator,
        }, components::{
            ADD_OUT_ID, CONSTANT_OUT_ID, Constant, REGISTER_OUT_ID, Register, Add,
        },
    };

    use crate::{
        PortType, VERYL_A_OUT_ID, VerylInputComponent, VerylRegisterComponent, combinational::{
            VERYL_Y_OUT_ID, VerylCombinationalComponent, 
        }, hybrid_simulator::{HybridSimulator, SyncRimToVerylMapping, VerylToSyncRimMapping},  veryl_bridge::VerylBridge, veryl_loader::load_veryl_simulator,
    };

    #[test]
    fn veryl_multiple_clk(){
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join("test_bridge_multiple_clock.veryl");

        let veryl_design = load_veryl_simulator(file.to_str().unwrap(), "MultipleClock").unwrap();

        let bridge = Rc::new(
            RefCell::new(
                VerylBridge::with_clock_reset(
                    veryl_design.simulator,
                    veryl_design.design_info, 
                    "clk_a", 
                    "rst_a"
                ).unwrap()
            )
        );

        let veryl_input_1 = Rc::new(
            VerylInputComponent::new(
                "veryl_input_1"
            )
        );
        let veryl_input_2 = Rc::new(
            VerylInputComponent::new(
                "veryl_input_2"
            )
        );

        let register_1 = Rc::new(
            VerylRegisterComponent::new(
                "register_1", 
                Input::new(
                    "veryl_input_1", 
                    VERYL_A_OUT_ID,
                )
            )
        );

        let register_2 = Rc::new(
            VerylRegisterComponent::new(
                "register_2", 
                Input::new(
                    "veryl_input_2", 
                    VERYL_A_OUT_ID,
                )
            )
        );

        let store = ComponentStore {
            store: vec![
                veryl_input_1,
                veryl_input_2,
                register_1,
                register_2,
            ]
        };

        let simulator = Simulator::new(store).unwrap();
        let veryl_to_syncrim = vec![
            VerylToSyncRimMapping {
                veryl_port: "d".into(),
                syncrim_component: "register_1".into(),
                syncrim_output: VERYL_Y_OUT_ID.into(),
                destination_type: PortType { width: 8, signed: false },
            },
            VerylToSyncRimMapping {
                veryl_port: "y".into(),
                syncrim_component: "register_2".into(),
                syncrim_output: VERYL_Y_OUT_ID.into(),
                destination_type: PortType { width: 8, signed: false },
            }
        ];
        let syncrim_to_veryl = vec![];

        let mut hybrid = HybridSimulator::new(simulator, bridge, veryl_to_syncrim, syncrim_to_veryl);

        let  input_1: Signal = 5.into();

        let port_type_1 = PortType {
            width: 8,
            signed: false,
        };
        hybrid.set_veryl_input("a", input_1.get_value(), port_type_1).unwrap();

        let input_2: Signal = 5.into();

        let port_type_2 = PortType {
            width: 8,
            signed: false,
        };

        hybrid.set_veryl_input("b", input_2.get_value(), port_type_2).unwrap();

        let reg_1_out = hybrid.syncrim.get_input_signal(&Input::new("register_1", VERYL_Y_OUT_ID));
        let reg_2_out = hybrid.syncrim.get_input_signal(&Input::new("register_2", VERYL_Y_OUT_ID));

        println!("Before clk_a");
        println!("Register 1: {:?}", reg_1_out);
        println!("Register 2: {:?}", reg_2_out);

        hybrid.clock().unwrap();

        let reg_1_out = hybrid.syncrim.get_input_signal(&Input::new("register_1", VERYL_Y_OUT_ID));
        let reg_2_out = hybrid.syncrim.get_input_signal(&Input::new("register_2", VERYL_Y_OUT_ID));

        println!("After clk_a");
        println!("Register 1: {:?}", reg_1_out);
        println!("Register 2: {:?}", reg_2_out);




    }

    #[test]
    fn hybrid_comb_chain_requires_multiple_settle_iterations(){
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join("test_bridge_two_comb.veryl");

        let veryl_design = load_veryl_simulator(file.to_str().unwrap(), "TestTwoComb").unwrap();

        let shared_bridge = Rc::new(
            RefCell::new(
                VerylBridge::new(
                    veryl_design.simulator,
                    veryl_design.design_info
                )
            )
        );


        let veryl_comb_1 = Rc::new(         
            VerylCombinationalComponent::new(
                "veryl_comb_1", 
                Input::new(
                    "first_add", 
                    ADD_OUT_ID
                )
            )          
        );

        let veryl_comb_2 = Rc::new(         
            VerylCombinationalComponent::new(
                "veryl_comb_2", 
                Input::new(
                    "second_add", 
                    ADD_OUT_ID
                )
            )          
        );
        let signal_a = SignalValue::Data(5);
        

        let constant_a = Rc::new(
            Constant::new(
                "constant_a",
                (1.0, 1.0), 
                signal_a
            )
        );
        let signal_b = SignalValue::Data(5);
        
        let constant_b = Rc::new(
            Constant::new(
                "constant_b",
                (1.0, 1.0), 
                signal_b
            )
        );

        let first_add = Rc::new(
            Add::new(
                "first_add", 
                (0.0, 0.0), 
                Input::new(
                    "constant_a", 
                    CONSTANT_OUT_ID
                ), 
                Input::new(
                    "constant_b",
                    CONSTANT_OUT_ID,
                )
            )
        );

        let signal_c = SignalValue::Data(4);

        let constant_c = Rc::new(
            Constant::new(
                "constant_c",
                (1.0, 1.0), 
                signal_c
            )
        );
    
        let second_add = Rc::new(
            Add::new(
                "second_add", 
                (0.0, 0.0), 
                Input::new(
                    "veryl_comb_1", 
                    VERYL_Y_OUT_ID
                ), 
                Input::new(
                    "constant_c",
                    CONSTANT_OUT_ID,
                )
            )
        );

        let signal_d = SignalValue::Data(5);

        let constant_d = Rc::new(
            Constant::new(
                "constant_d",
                (0.0, 0.0),
                signal_d,
            )
        );

        let third_add = Rc::new(
            Add::new(
                "third_add",
                (0.0, 0.0),
                Input::new(
                    "veryl_comb_2",
                    VERYL_Y_OUT_ID,
                ),
                Input::new(
                    "constant_d", 
                    CONSTANT_OUT_ID,
                )
            )
        );
        let veryl_comb_3 = Rc::new(
            VerylCombinationalComponent::new(
                "veryl_comb_3",
                Input::new(
                    "third_add", 
                    ADD_OUT_ID,
                )
            )
        );

        let signal_e = SignalValue::Data(5);
        let constant_e = Rc::new(
            Constant::new(
                "constant_e", 
                (0.0, 0.0), 
                signal_e,
            )
        );

        let fourth_add = Rc::new(
            Add::new(
                "fourth_add", 
                (0.0, 0.0), 
                Input::new(
                    "veryl_comb_3",
                    VERYL_Y_OUT_ID,
                ),
                Input::new(
                    "constant_e",
                    CONSTANT_OUT_ID,
                )
            )
        );
        let veryl_comb_4 = Rc::new(
            VerylCombinationalComponent::new(
                "veryl_comb_4",
                Input::new(
                    "fourth_add", 
                    ADD_OUT_ID,
                )
            )
        );

        let store = ComponentStore {
            store: vec![
                veryl_comb_1,
                veryl_comb_2,
                veryl_comb_3,
                veryl_comb_4,
                constant_a,
                constant_b,
                constant_c,
                constant_d,
                constant_e,
                first_add,
                second_add,
                third_add,
                fourth_add,
            ],
        };

        let simulator = Simulator::new(store).unwrap();

        let veryl_to_syncrim_mappings = vec![
            VerylToSyncRimMapping {
                veryl_port: "y1".into(),
                syncrim_component: "veryl_comb_1".into(),
                syncrim_output: VERYL_Y_OUT_ID.into(),
                destination_type: PortType { width: 8, signed: false },
            },
            VerylToSyncRimMapping {
                veryl_port: "y2".into(),
                syncrim_component: "veryl_comb_2".into(),
                syncrim_output: VERYL_Y_OUT_ID.into(),
                destination_type: PortType { width: 8, signed: false },
            },
            VerylToSyncRimMapping {
                veryl_port: "y3".into(),
                syncrim_component: "veryl_comb_3".into(),
                syncrim_output: VERYL_Y_OUT_ID.into(),
                destination_type: PortType { width: 8, signed: false },
            },
            VerylToSyncRimMapping {
                veryl_port: "y4".into(),
                syncrim_component: "veryl_comb_4".into(),
                syncrim_output: VERYL_Y_OUT_ID.into(),
                destination_type: PortType { width: 8, signed: false },
            }
        ];

        let syncrim_to_veryl_mappings = vec![
            SyncRimToVerylMapping {
                syncrim_component: "first_add".to_string(),
                syncrim_output: ADD_OUT_ID.to_string(),
                source_type: PortType {
                    width: 8,
                    signed: false,
                },
                veryl_port: "a1".to_string(),
            },
            SyncRimToVerylMapping {
                syncrim_component: "second_add".to_string(),
                syncrim_output: ADD_OUT_ID.to_string(),
                source_type: PortType {
                    width: 8,
                    signed: false,
                },
                veryl_port: "a2".to_string(),
            },
            SyncRimToVerylMapping {
                syncrim_component: "third_add".to_string(),
                syncrim_output: ADD_OUT_ID.to_string(),
                source_type: PortType {
                    width: 8,
                    signed: false,
                },
                veryl_port: "a3".to_string(),
            },
            SyncRimToVerylMapping {
                syncrim_component: "fourth_add".to_string(),
                syncrim_output: ADD_OUT_ID.to_string(),
                source_type: PortType {
                    width: 8,
                    signed: false,
                },
                veryl_port: "a4".to_string(),
            }
            
        ];

        let mut hybrid = HybridSimulator::new(simulator, shared_bridge.clone(), veryl_to_syncrim_mappings, syncrim_to_veryl_mappings);

        
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("first_add", ADD_OUT_ID)).get_value(), SignalValue::Data(10));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_comb_1", VERYL_Y_OUT_ID)).get_value(), SignalValue::Data(0));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("second_add", ADD_OUT_ID)).get_value(), SignalValue::Data(4));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_comb_2", VERYL_Y_OUT_ID)).get_value(), SignalValue::Data(0));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("third_add", ADD_OUT_ID)).get_value(), SignalValue::Data(5));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_comb_3", VERYL_Y_OUT_ID)).get_value(), SignalValue::Data(0));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("fourth_add", ADD_OUT_ID)).get_value(), SignalValue::Data(5));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_comb_4", VERYL_Y_OUT_ID)).get_value(), SignalValue::Data(0));

        hybrid.settle_combinational().unwrap();
        
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("first_add", ADD_OUT_ID)).get_value(), SignalValue::Data(10));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_comb_1", VERYL_Y_OUT_ID)).get_value(), SignalValue::Data(11));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("second_add", ADD_OUT_ID)).get_value(), SignalValue::Data(15));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_comb_2", VERYL_Y_OUT_ID)).get_value(), SignalValue::Data(16));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("third_add", ADD_OUT_ID)).get_value(), SignalValue::Data(21));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_comb_3", VERYL_Y_OUT_ID)).get_value(), SignalValue::Data(22));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("fourth_add", ADD_OUT_ID)).get_value(), SignalValue::Data(27));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_comb_4", VERYL_Y_OUT_ID)).get_value(), SignalValue::Data(28));

    }
    
    #[test]
    fn hybrid_sequential_pipeline_preserves_clock_boundries(){
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join("test_bridge_register.veryl");

        let veryl_design = load_veryl_simulator(file.to_str().unwrap(), "TestRegister").unwrap();

        let shared_bridge = Rc::new(
            RefCell::new(
                VerylBridge::with_clock_reset(
                    veryl_design.simulator,
                    veryl_design.design_info,
                    "clk",
                    "rst",
                ).unwrap()
            )
        );

        let constant_reg = SignalValue::Data(10);
        
        let constant_register = Rc::new(
            Constant::new(
                "constant_register", 
                (0.0, 0.0), 
                constant_reg,
            )
        );

        let register_1 = Rc::new(
            Register::new(
                "register_1", 
                (0.0, 0.0), 
                Input::new(
                    "constant_register",
                    CONSTANT_OUT_ID,
                )    
            )
        );

        let const_a = SignalValue::Data(0);

        let constant_1 = Rc::new(
            Constant::new(
                "constant_1", 
                (0.0, 0.0), 
                const_a,    
            )
        );



        let add_1 = Rc::new(
            Add::new(
                "add_1",
                (0.0, 0.0),
                Input::new(
                    "register_1", 
                    REGISTER_OUT_ID,
                ),
                Input::new(
                    "constant_1", 
                    CONSTANT_OUT_ID,
                )
            )
        );

        let veryl_register = Rc::new(
            VerylRegisterComponent::new(
                "veryl_register", 
                Input::new(
                    "add_1",
                    ADD_OUT_ID,
                )
            )
        );

        let constant_add_2 = SignalValue::Data(0);

        let constant_add_2 = Rc::new(
            Constant::new(
                "constant_add_2", 
                (0.0, 0.0), 
                constant_add_2,
            )
        );

        let add_2 = Rc::new(
            Add::new(
                "add_2",
                (0.0, 0.0),
                Input::new(
                    "veryl_register", 
                    VERYL_Y_OUT_ID,
                ),
                Input::new(
                    "constant_add_2",
                    CONSTANT_OUT_ID,
                )
            )
        );

        let register_2 = Rc::new(
            Register::new(
                "register_2", 
                (0.0, 0.0), 
                Input::new(
                    "add_2", 
                    ADD_OUT_ID
                )
            )
        );

        let store = ComponentStore {
            store: vec![
                constant_register,
                register_1,
                register_2,
                veryl_register,
                add_1,
                add_2,
                constant_1,
                constant_add_2,
            ]
        };

        let simulator = Simulator::new(store).unwrap();

        let veryl_mappings = vec![
            VerylToSyncRimMapping {
                veryl_port: "y".into(),
                syncrim_component: "veryl_register".into(),
                syncrim_output: VERYL_Y_OUT_ID.into(),
                destination_type: PortType { width: 8, signed: false }
            }
        ];

        let syncrim_mappings = vec![
            SyncRimToVerylMapping {
                syncrim_component: "add_1".into(),
                syncrim_output: ADD_OUT_ID.into(),
                source_type: PortType {
                    width: 8,
                    signed: false,
                },
                veryl_port: "d".into(),
            }
        ];

        let mut hybrid = HybridSimulator::new(simulator, shared_bridge, veryl_mappings, syncrim_mappings);

        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("register_1", REGISTER_OUT_ID)).get_value(), SignalValue::Data(0));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("register_2", REGISTER_OUT_ID)).get_value(), SignalValue::Data(0));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_register", VERYL_Y_OUT_ID)).get_value(), SignalValue::Data(0));

        assert_eq!(hybrid.cycle, 0);

        hybrid.clock().unwrap();

        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("register_1", REGISTER_OUT_ID)).get_value(), SignalValue::Data(10));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("register_2", REGISTER_OUT_ID)).get_value(), SignalValue::Data(0));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_register", VERYL_Y_OUT_ID)).get_value(), SignalValue::Data(0));

        assert_eq!(hybrid.cycle, 1);

        hybrid.clock().unwrap();

        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("register_1", REGISTER_OUT_ID)).get_value(), SignalValue::Data(10));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_register", VERYL_Y_OUT_ID)).get_value(), SignalValue::Data(10));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("register_2", REGISTER_OUT_ID)).get_value(), SignalValue::Data(0));

        assert_eq!(hybrid.cycle, 2);

        hybrid.clock().unwrap();
     
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("register_1", REGISTER_OUT_ID)).get_value(), SignalValue::Data(10));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_register", VERYL_Y_OUT_ID)).get_value(), SignalValue::Data(10));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("register_2", REGISTER_OUT_ID)).get_value(), SignalValue::Data(10));
        
        assert_eq!(hybrid.cycle, 3);

    }
    #[test]
    fn veryl_comb_to_syncrim_seq(){
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join("test_bridge.veryl");

        let veryl_design = load_veryl_simulator(file.to_str().unwrap(), "TestBridge").unwrap();

        let shared_bridge = Rc::new(
            RefCell::new(
                VerylBridge::new(veryl_design.simulator, veryl_design.design_info)
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
                ))
        );

        let syncrim_register = Rc::new(
            Register::new(
                "syncrim_register", 
                (0.0, 0.0), 
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
                syncrim_register,
            ]
        };

        let simulator = Simulator::new(store).unwrap();

        let veryl_mappings = vec![
            VerylToSyncRimMapping {
                veryl_port: "y".into(),
                syncrim_component: "veryl_comb".into(),
                syncrim_output: VERYL_Y_OUT_ID.into(),
                destination_type: PortType { width: 8, signed: false },
            }
        ];

        let syncrim_mappings = vec![];

        let mut hybrid = HybridSimulator::new(simulator, shared_bridge.clone(), veryl_mappings, syncrim_mappings);

        let signal= SignalValue::Data(10);

        let port_type = PortType {
            width: 8,
            signed: false,
        };

        hybrid.set_veryl_input("a", signal, port_type).unwrap();
        hybrid.initialize_state().unwrap();
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_comb", VERYL_Y_OUT_ID)).get_value(), SignalValue::Data(11));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("syncrim_register", REGISTER_OUT_ID)).get_value(), SignalValue::Data(0));
        hybrid.clock().unwrap();
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("syncrim_register", REGISTER_OUT_ID)).get_value(), SignalValue::Data(11));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_comb", VERYL_Y_OUT_ID)).get_value(), SignalValue::Data(11));


    }
    #[test]
    fn syncrim_comb_to_veryl_seq(){
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join("test_bridge_register.veryl");

        let veryl_design = load_veryl_simulator(file.to_str().unwrap(), "TestRegister").unwrap();

        let shared_bridge = Rc::new(
            RefCell::new(
                VerylBridge::with_clock_reset(
                    veryl_design.simulator,
                    veryl_design.design_info,
                    "clk", 
                    "rst",
                ).unwrap()
            )
        );
        let input_1 = SignalValue::Data(5);

        let constant_a = Rc::new(
            Constant::new(
                "constant_a", 
                (1.0, 1.0),
                input_1,
            )
        );

        let input_2 = SignalValue::Data(5);

        let constant_b = Rc::new(
            Constant::new(
                "constant_b", 
                (1.0, 1.0),
                input_2,
            )
        );

        let add_component = Rc::new(
            Add::new(
                "add_component", 
                (0.0, 0.0), 
                Input::new("constant_a", CONSTANT_OUT_ID), 
                Input::new("constant_b", CONSTANT_OUT_ID),
            )
        );

        let register_component = Rc::new(
            VerylRegisterComponent::new(
                "register_component", 
                Input::new("add_component", ADD_OUT_ID),
            )
        );

        let store = ComponentStore {
            store: vec![
                constant_a,
                constant_b,
                add_component,
                register_component,
            ]
        };

        let simulator = Simulator::new(store).unwrap();

        let veryl_mappings = vec![
            VerylToSyncRimMapping {
                veryl_port: "y".into(),
                syncrim_component: "register_component".into(),
                syncrim_output: VERYL_Y_OUT_ID.into(),
                destination_type: PortType { width: 8, signed: false },
            }
        ];

        let syncrim_mappings = vec![
            SyncRimToVerylMapping {
                syncrim_component: "add_component".into(),
                syncrim_output: ADD_OUT_ID.into(),
                source_type: PortType {
                    width: 8,
                    signed: false,
                },
                veryl_port: "d".into(),

            }
        ];

        let mut hybrid = HybridSimulator::new(simulator, shared_bridge.clone(), veryl_mappings, syncrim_mappings);

        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("add_component", ADD_OUT_ID)).get_value(), SignalValue::Data(10));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("register_component", VERYL_Y_OUT_ID)).get_value(), SignalValue::Data(0));

        hybrid.clock().unwrap();

        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("add_component", ADD_OUT_ID)).get_value(), SignalValue::Data(10));
        assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("register_component", VERYL_Y_OUT_ID)).get_value(), SignalValue::Data(10));
    }
}