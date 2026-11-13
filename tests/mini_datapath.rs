use std::{path::PathBuf, rc::Rc};

use hybrid_sim::{MiniDatapathComponent, PortType, RESULT_OUT_ID, VERYL_INPUT_OUT_ID, VerylBridge, VerylInputComponent, hybrid_simulator::{HybridSimulator, VerylToSyncRimMapping}, load_veryl_simulator};
use syncrim::{common::{ComponentStore, Input, Simulator}, components::{ADD_OUT_ID, Add, CONSTANT_OUT_ID, Constant, REGISTER_OUT_ID, Register}, signal::SignalValue};



#[test]
fn mini_datapath_runs_hybrid_simulation(){
    let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join("mini_datapath.veryl");

    let veryl_design = load_veryl_simulator(file.to_str().unwrap(), "MiniDatapath").unwrap();

    let bridge = VerylBridge::new(veryl_design.simulator, veryl_design.design_info).unwrap();

    let const_signal = SignalValue::Data(1);

    let constant = Rc::new(
        Constant::new(
            "constant", 
            (0.0, 0.0), 
            const_signal
        )
    );

    let input_a = Rc::new(
        VerylInputComponent::new(
            "input_a",
        )
    );
    let input_b = Rc::new(
        VerylInputComponent::new(
            "input_b",
        )
    );
    let input_op = Rc::new(
        VerylInputComponent::new(
            "input_op",
        )
    );
    let input_enable = Rc::new(
        VerylInputComponent::new(
            "input_enable",
        )
    );
    let veryl_component = Rc::new(
        MiniDatapathComponent::new(
            "veryl_component", 
            Input::new(
                "input_a",
                VERYL_INPUT_OUT_ID,
            ), 
            Input::new(
                "input_b",
                VERYL_INPUT_OUT_ID,
            ), 
            Input::new(
                "input_op",
                VERYL_INPUT_OUT_ID,
            ),
            Input::new(
                "input_enable",
                VERYL_INPUT_OUT_ID,
            ),
        )
    );
    let add = Rc::new(
        Add::new(
            "add",
            (0.0, 0.0),
            Input::new(
                "veryl_component", 
                RESULT_OUT_ID,
            ),
            Input::new(
                "constant",
                CONSTANT_OUT_ID,
            ),
        )
    );

    let register = Rc::new(
        Register::new(
            "register", 
            (0.0, 0.0), 
            Input::new(
                "add", 
                ADD_OUT_ID,
            )
        )
    );

    let store = ComponentStore {
        store: vec![
            constant,
            input_a,
            input_b,
            input_enable,
            input_op,
            veryl_component,
            add,
            register,
        ]
    };
    let veryl_to_syncrim = vec![
        VerylToSyncRimMapping{
            veryl_port: "result".into(),
            syncrim_component: "veryl_component".into(),
            syncrim_output: RESULT_OUT_ID.into(),
            destination_type: PortType{
                width: 8,
                signed: false,
            }
        }
    ];
    let simulator = Simulator::new(store).unwrap();
    let syncrim_to_veryl = vec![];

    let mut hybrid = HybridSimulator::new(simulator, bridge, veryl_to_syncrim, syncrim_to_veryl);
    // Initialize reset
    hybrid.initialize_state().unwrap();

    hybrid.reset().unwrap();

    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_component", RESULT_OUT_ID)).get_value(), SignalValue::Data(0));
    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("add", ADD_OUT_ID)).get_value(), SignalValue::Data(1));
    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("register", REGISTER_OUT_ID)).get_value(), SignalValue::Data(0));

    // ADD 10 + 5
    hybrid.set_veryl_input("a", SignalValue::Data(10), PortType { width: 8, signed: false }).unwrap();
    hybrid.set_veryl_input("b", SignalValue::Data(5), PortType { width: 8, signed: false }).unwrap();
    hybrid.set_veryl_input("op", SignalValue::Data(0), PortType { width: 2, signed: false }).unwrap();
    hybrid.set_veryl_input("enable", SignalValue::Data(1), PortType { width: 1, signed: false }).unwrap();

    hybrid.clock().unwrap();

    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_component", RESULT_OUT_ID)).get_value(), SignalValue::Data(15));
    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("add", ADD_OUT_ID)).get_value(), SignalValue::Data(16));
    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("register", REGISTER_OUT_ID)).get_value(), SignalValue::Data(1));
    
    // SUB 20 - 3
    hybrid.set_veryl_input("a", SignalValue::Data(20), PortType { width: 8, signed: false }).unwrap();
    hybrid.set_veryl_input("b", SignalValue::Data(3), PortType { width: 8, signed: false }).unwrap();
    hybrid.set_veryl_input("op", SignalValue::Data(1), PortType { width: 2, signed: false }).unwrap();
    hybrid.set_veryl_input("enable", SignalValue::Data(1), PortType { width: 1, signed: false }).unwrap();

    hybrid.clock().unwrap();

    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_component", RESULT_OUT_ID)).get_value(), SignalValue::Data(17));
    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("add", ADD_OUT_ID)).get_value(), SignalValue::Data(18));
    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("register", REGISTER_OUT_ID)).get_value(), SignalValue::Data(16));
    
    // enable = 0
    hybrid.set_veryl_input("a", SignalValue::Data(100), PortType { width: 8, signed: false }).unwrap();
    hybrid.set_veryl_input("b", SignalValue::Data(50), PortType { width: 8, signed: false }).unwrap();
    hybrid.set_veryl_input("op", SignalValue::Data(0), PortType { width: 2, signed: false }).unwrap();
    hybrid.set_veryl_input("enable", SignalValue::Data(0), PortType { width: 1, signed: false }).unwrap();

    hybrid.clock().unwrap();

    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_component", RESULT_OUT_ID)).get_value(), SignalValue::Data(17));
    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("add", ADD_OUT_ID)).get_value(), SignalValue::Data(18));
    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("register", REGISTER_OUT_ID)).get_value(), SignalValue::Data(18));
    
    // AND 10 & 10
    hybrid.set_veryl_input("a", SignalValue::Data(10), PortType { width: 8, signed: false }).unwrap();
    hybrid.set_veryl_input("b", SignalValue::Data(10), PortType { width: 8, signed: false }).unwrap();
    hybrid.set_veryl_input("op", SignalValue::Data(2), PortType { width: 2, signed: false }).unwrap();
    hybrid.set_veryl_input("enable", SignalValue::Data(1), PortType { width: 1, signed: false }).unwrap();

    hybrid.clock().unwrap();

    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_component", RESULT_OUT_ID)).get_value(), SignalValue::Data(10));
    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("add", ADD_OUT_ID)).get_value(), SignalValue::Data(11));
    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("register", REGISTER_OUT_ID)).get_value(), SignalValue::Data(18));

    hybrid.set_veryl_input("a", SignalValue::Data(20), PortType { width: 8, signed: false }).unwrap();
    hybrid.set_veryl_input("b", SignalValue::Data(10), PortType { width: 8, signed: false }).unwrap();
    hybrid.set_veryl_input("op", SignalValue::Data(3), PortType { width: 2, signed: false }).unwrap();
    hybrid.set_veryl_input("enable", SignalValue::Data(1), PortType { width: 1, signed: false }).unwrap();
    
    // OR 20 | 10
    hybrid.clock().unwrap();

    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_component", RESULT_OUT_ID)).get_value(), SignalValue::Data(30));
    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("add", ADD_OUT_ID)).get_value(), SignalValue::Data(31));
    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("register", REGISTER_OUT_ID)).get_value(), SignalValue::Data(11));

    hybrid.reset().unwrap();
    
    // Reset
    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("veryl_component", RESULT_OUT_ID)).get_value(), SignalValue::Data(0));
    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("add", ADD_OUT_ID)).get_value(), SignalValue::Data(1));
    assert_eq!(hybrid.syncrim.get_input_signal(&Input::new("register", REGISTER_OUT_ID)).get_value(), SignalValue::Data(0));

    
}