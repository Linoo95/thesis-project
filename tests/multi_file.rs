use std::rc::Rc;

use hybrid_sim::{PortType, RESULT_OUT_ID, VERYL_INPUT_OUT_ID, VerylInputComponent, hybrid_simulator::{HybridSimulator, SyncRimToVerylMapping, VerylToSyncRimMapping}, load_veryl_bridge, multi_file::{VERYL_ALU_OUT_ID, VERYL_Q_OUT_ID, VERYL_RESULT_OUT_ID, VerylAluComponent, VerylRegisterComponent, VerylTopComponent}};
use syncrim::{common::{ComponentStore, Input, Simulator}, components::REGISTER_OUT_ID, signal::SignalValue};





#[test]
fn multi_veryl_file_test(){

    let veryl_bridge = load_veryl_bridge("multi_file", "MultiFileTop").unwrap();

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

    let top_component = Rc::new(
        VerylTopComponent::new(
            "top_component", 
            Input::new("input_a", VERYL_INPUT_OUT_ID), 
            Input::new("input_b", VERYL_INPUT_OUT_ID), 
            Input::new("input_op", VERYL_INPUT_OUT_ID), 
            Input::new("input_enable", VERYL_INPUT_OUT_ID),
        )
    );

    let reg_component = Rc::new(
        VerylRegisterComponent::new(
            "reg_component", 
            Input::new(
                "input_enable", 
                VERYL_INPUT_OUT_ID,
            ), 
            Input::new(
                "input_d", 
                VERYL_INPUT_OUT_ID,
            )
        )
    );

    let alu_component  = Rc::new(
        VerylAluComponent::new(
            "alu_component", 
            Input::new(
                "input_a", 
                VERYL_INPUT_OUT_ID
            ), 
            Input::new(
                "input_b", 
                VERYL_INPUT_OUT_ID,
            ), 
            Input::new(
                "input_op", 
                VERYL_INPUT_OUT_ID,
            )
        )
    );

    let store = ComponentStore{
        store: vec![
            input_a,
            input_b,
            input_enable,
            input_op,
            top_component,
            reg_component,
            alu_component,
        ]
    };

    let simulator = Simulator::new(store).unwrap();

    let veryl_to_syncrim = vec![
        VerylToSyncRimMapping{
            veryl_port: "result".into(),
            syncrim_component: "top_component".into(),
            syncrim_output: RESULT_OUT_ID.into(),
            destination_type: PortType { width: 8, signed: false }
        },
        VerylToSyncRimMapping{
            veryl_port: "alu_out".into(),
            syncrim_component: "alu_component".into(),
            syncrim_output: VERYL_ALU_OUT_ID.into(),
            destination_type: PortType { width: 8, signed: false }
        },
        VerylToSyncRimMapping{
            veryl_port: "reg_out".into(),
            syncrim_component: "reg_component".into(),
            syncrim_output: VERYL_Q_OUT_ID.into(),
            destination_type: PortType { width: 8, signed: false }
        }

    ];
    let syncrim_to_veryl = vec![
        // SyncRimToVerylMapping{
        //     syncrim_component: "input_a".into(),
        //     syncrim_output: VERYL_INPUT_OUT_ID.into(),
        //     source_type: PortType { width: 8, signed: false },
        //     veryl_port: "a".into(),
        // },
        // SyncRimToVerylMapping{
        //     syncrim_component: "input_b".into(),
        //     syncrim_output: VERYL_INPUT_OUT_ID.into(),
        //     source_type: PortType { width: 8, signed: false },
        //     veryl_port: "b".into(),
        // },
        // SyncRimToVerylMapping{
        //     syncrim_component: "input_op".into(),
        //     syncrim_output: VERYL_INPUT_OUT_ID.into(),
        //     source_type: PortType { width: 2, signed: false },
        //     veryl_port: "op".into(),
        // },
        // SyncRimToVerylMapping{
        //     syncrim_component: "input_enable".into(),
        //     syncrim_output: VERYL_INPUT_OUT_ID.into(),
        //     source_type: PortType { width: 1, signed: false },
        //     veryl_port: "enable".into(),
        // }
    ];

    let mut hybrid = HybridSimulator::new(simulator, veryl_bridge, veryl_to_syncrim, syncrim_to_veryl);

    hybrid.initialize_state().unwrap();
    hybrid.set_veryl_input("a", SignalValue::Data(5), PortType { width: 8, signed: false }).unwrap();
    hybrid.set_veryl_input("b", SignalValue::Data(5), PortType { width: 8, signed: false }).unwrap();
    hybrid.set_veryl_input("op", SignalValue::Data(0), PortType { width: 2, signed: false }).unwrap();
    hybrid.set_veryl_input("enable", SignalValue::Data(1), PortType { width: 1, signed: false }).unwrap();

    // hybrid.syncrim.set_out_value("input_a", VERYL_INPUT_OUT_ID, SignalValue::Data(10));
    // hybrid.syncrim.set_out_value("input_b", VERYL_INPUT_OUT_ID, SignalValue::Data(5));
    // hybrid.syncrim.set_out_value("input_op", VERYL_INPUT_OUT_ID, SignalValue::Data(0));
    // hybrid.syncrim.set_out_value("input_enable", VERYL_INPUT_OUT_ID, SignalValue::Data(1));
    hybrid.settle_combinational().unwrap();
    println!("Alu output: {:?}", hybrid.syncrim.get_input_value(&Input::new("alu_component", VERYL_ALU_OUT_ID)));
    println!("Register output: {:?}", hybrid.syncrim.get_input_value(&Input::new("reg_component", VERYL_Q_OUT_ID)));
    println!("Result: {:?}", hybrid.syncrim.get_input_value(&Input::new("top_component", VERYL_RESULT_OUT_ID)));
    hybrid.clock().unwrap();
    println!("Alu output: {:?}", hybrid.syncrim.get_input_value(&Input::new("alu_component", VERYL_ALU_OUT_ID)));
    println!("Register output: {:?}", hybrid.syncrim.get_input_value(&Input::new("reg_component", VERYL_Q_OUT_ID)));
    println!("Result: {:?}", hybrid.syncrim.get_input_value(&Input::new("top_component", VERYL_RESULT_OUT_ID)));
}