use std::{cell::RefCell, rc::Rc, str::FromStr};

use syncrim::signal::{SignalValue};
use veryl_analyzer::ir::VarPath;
use veryl_simulator::{Simulator, ir::{Event, Value}};

use crate::{PortDirection, PortType, VerylDesignInfo, VerylPortInfo, convert_syncrim_to_veryl};

pub type SharedVerylBridge = Rc<RefCell<VerylBridge>>;

pub struct VerylBridge {
    simulator: Simulator,
    pub design_info: VerylDesignInfo,
    clock: Option<Event>,
    reset: Option<Event>,
}

impl VerylBridge {

    pub fn new( simulator: Simulator, design_info:VerylDesignInfo) -> Self {
        Self {
            simulator,
            design_info,
            clock: None,
            reset: None,
        }
    }

    pub fn with_clock_reset(
        simulator: Simulator,
        design_info: VerylDesignInfo,
        clock_port: &str,
        reset_port: &str,
    ) -> Result<Self, String> {
        let clock = simulator.get_clock(clock_port).ok_or_else(|| {format!("Could not find veryl clock port {clock_port}")})?;
        let reset = simulator.get_reset(reset_port).ok_or_else(|| {format!("Could not find veryl reset port {reset_port}")})?;
        Ok(Self {
            simulator,
            design_info,
            clock: Some(clock),
            reset: Some(reset),
        })
    }

    pub fn input_ports(&self) -> impl Iterator<Item = &VerylPortInfo> {
        self.design_info.ports.iter().filter(|port| port.direction == PortDirection::Input)
    }

    pub fn output_ports(&self) -> impl Iterator<Item = &VerylPortInfo> {
        self.design_info.ports.iter().filter(|port| port.direction == PortDirection::Output)
    }
    
    pub fn set_input(
        &mut self,
        port: &str,
        value: SignalValue,
        source_type: PortType,
    ) -> Result<(), String> {
        let port_info= self.port_info(port)?;

        if port_info.direction != PortDirection::Input{
            return Err(format!("Cannot write to port: '{port}': port is {:?}", port_info.direction));
        }
        let value = convert_syncrim_to_veryl(
            value, 
            source_type, 
            port_info.port_type,
        )?;
        self.simulator.set(port, value);
        Ok(())
    }

    pub fn port_info(&self, port: &str) -> Result<&VerylPortInfo, String> {
        self.design_info.ports.iter().find(|p| p.name == port).ok_or_else(|| format!("Veryl port '{port}' was not found"))
    }

    pub fn port_type(
        &self,
        port: &str,
    )-> Result<PortType, String>{
        Ok(self.port_info(port)?.port_type)
    }

    pub fn port_direction(
        &self,
        port: &str,
    )-> Result<PortDirection, String>{
        Ok(self.port_info(port)?.direction)
    }

    pub fn clock(&mut self) -> Result<(), String> {
        let clock = self.clock.as_ref().ok_or("VerylBridge has no clock")?;
        self.simulator.step(clock);

        Ok(())
    }

    pub fn has_clock(&self) -> bool {
        self.clock.is_some()
    }

    pub fn reset(&mut self) -> Result<(), String>{
        let clock = self.clock.as_ref().ok_or("VerylBridge has no clock")?;
        let reset = self.reset.as_ref().ok_or("VerylBridge has no reset")?;
        self.simulator.step_reset(clock, reset);
        Ok(())
    }

    pub fn read_value(
        &mut self,
        port: &str,
    ) -> Result<Value, String> {
        let port_info = self.port_info(port)?;

        if port_info.direction != PortDirection::Output{
            return Err(format!("Cannot read Veryl output '{port}': port is {:?}", port_info.direction));
        }
        self.simulator.get(port).ok_or_else(|| format!("Could not read Veryl port '{port}'"))
    }

}






#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use syncrim::{signal::{ Signal, SignalValue}};
use veryl_simulator::{ir::Value};



    use crate::{PortDirection, PortType, veryl_bridge::VerylBridge};

    use crate::veryl_loader::load_veryl_simulator;



    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name)
    }
    #[test]
    fn get_ir_info(){
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join("test_bridge_two_comb.veryl");

        let veryl_design = load_veryl_simulator(file.to_str().unwrap(), "TestTwoComb").unwrap();

        let bridge = VerylBridge::new(veryl_design.simulator, veryl_design.design_info);
        
        // for port in bridge.input_ports() {
        //     println!("Veryl input: {:?}", port)
        // };
        // for port in bridge.output_ports() {
        //     println!("Veryl output: {:?}", port)
        // };

        let info = bridge.port_info("a1").unwrap();
        println!("Port a1: {:?}", info);
        
    }
    #[test]
    fn veryl_comb_to_seq(){
        let file = fixture("test_bridge_comb_seq.veryl");

        let veryl_design = load_veryl_simulator(file.to_str().unwrap(), "TestCombSeq").unwrap();

        let mut bridge = VerylBridge::with_clock_reset(veryl_design.simulator, veryl_design.design_info , "clk", "rst").unwrap();

        bridge.reset().unwrap();

        let y = bridge.read_value("y").unwrap();

        match y {
            Value::U64(v) => {
                assert_eq!(v.payload, 0);
            }
            _ => {
                panic!("Expected Value::U64")
            }
        };

        let signal: Signal = 10.into();

        let port_type = PortType {
            width: 8,
            signed: false,
        };

        bridge.set_input("a", signal.get_value(), port_type).unwrap();

        bridge.clock().unwrap();
        let y = bridge.read_value("y").unwrap();
        match y {
            Value::U64(v) => {
                assert_eq!(v.payload, 11);
            }
            _ => {
                panic!("Expected Value::U64")
            }
        };

        let signal_2: Signal = 5.into();

        let port_type_2 = PortType {
            width: 8,
            signed: false,
        };

        bridge.set_input("a", signal_2.get_value(), port_type_2).unwrap();

        let y = bridge.read_value("y").unwrap();
        match y {
            Value::U64(v) => {
                assert_eq!(v.payload, 11);
            }
            _ => {
                panic!("Expected Value::U64")
            }
        };


        bridge.clock().unwrap();
        let y = bridge.read_value("y").unwrap();
        match y {
            Value::U64(v) => {
                assert_eq!(v.payload, 6);
            }
            _ => {
                panic!("Expected Value::U64")
            }
        };

    }

    #[test]
    fn syncrim_unsigned_signal_can_drive_veryl_input(){

        let file = fixture("test_bridge.veryl");
        let veryl_design = load_veryl_simulator(file.to_str().unwrap(), "TestBridge").unwrap();

        let mut bridge = VerylBridge::new(veryl_design.simulator, veryl_design.design_info);

        let signal = SignalValue::Data(10);

        let port_type = PortType {
            width: 8,
            signed: false,
        };

        bridge.set_input("a", signal, port_type).unwrap();

        let output = bridge.read_value("y").unwrap();

        match output {
            Value::U64(v) => {
                assert_eq!(v.payload, 11);
                assert_eq!(v.width, 8);
                assert_eq!(v.signed, false);
                assert_eq!(v.mask_xz, 0);
            }
            _ => {
                panic!("Expected Value::U64")
            }
        }
        
    }

}