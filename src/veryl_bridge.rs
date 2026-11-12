use std::{collections::HashMap};

use syncrim::signal::{SignalValue};

use veryl_analyzer::symbol::{Direction, TypeKind};
use veryl_simulator::{Simulator, ir::{Event, Value}};

use crate::{PortType, VerylDesignInfo, VerylPortInfo, convert_syncrim_to_veryl};

//pub type SharedVerylBridge = Rc<RefCell<VerylBridge>>;

pub struct VerylBridge {
    simulator: Simulator,
    design_info: VerylDesignInfo,
    clocks: HashMap<String, Event>,
    resets: HashMap<String, Event>,
}

impl VerylBridge {
    pub fn new(
        simulator: Simulator,
        design_info: VerylDesignInfo,
    ) -> Result<Self, String> {
        let mut clocks = HashMap::new();
        let mut resets = HashMap::new();
        for port in &design_info.ports {
            if port.type_kind.is_clock(){
                if port.type_kind != TypeKind::Clock {
                    return Err(format!("Unsupported clock type: {:?}", port.type_kind));
                }
                let event = simulator.get_clock(&port.name).ok_or_else(|| {format!("Could not find veryl clock port {}", port.name)})?;
                clocks.insert(port.name.clone(), event); 
            }
            if port.type_kind.is_reset(){
                if port.type_kind != TypeKind::Reset{
                    return Err(format!("Unsupported reset type: {:?}", port.type_kind));
                }
                let event = simulator.get_reset(&port.name).ok_or_else(|| {format!("Could not find veryl reset port {}", port.name)})?;
                resets.insert(port.name.clone(), event);       
            }
        }
        Ok(Self {
            simulator,
            design_info,
            clocks,
            resets,
        })
    }

    pub fn data_input_ports(&self) -> impl Iterator<Item = &VerylPortInfo> {
        // Ta bork clock och reset?
        self.design_info.ports.iter().filter(|port| port.direction == Direction::Input && !port.type_kind.is_clock() && !port.type_kind.is_reset())
    }

    pub fn output_ports(&self) -> impl Iterator<Item = &VerylPortInfo> {
        self.design_info.ports.iter().filter(|port| port.direction == Direction::Output && !port.type_kind.is_clock() && !port.type_kind.is_reset())
    }
    
    pub fn set_input(
        &mut self,
        port: &str,
        value: SignalValue,
        source_type: PortType,
    ) -> Result<(), String> {
        let port_info= self.port_info(port)?;

        if port_info.direction != Direction::Input{
            return Err(format!("Cannot write to port: '{port}': port is {:?}", port_info.direction));
        }

        if port_info.type_kind.is_clock() || port_info.type_kind.is_reset(){
            return Err(format!("Cannot write clock/reset port '{port}' as a data input"));
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


    pub fn clock(&mut self, name: &str) -> Result<(), String> {
        let clock = self.clocks.get(name).ok_or_else(|| {
            format!("VerylBridge has no clock named: '{name}'")
        })?;
        self.simulator.step(clock);

        Ok(())
    }

    pub fn clock_default(&mut self) -> Result<(), String> {
        match self.clock_count(){
            0 => {
                return Err("VerylBridge has no clock".into());
            }
            1 => {
                let name = self.clocks.keys().next().unwrap().clone();
                self.clock(&name)
            }
            n => {
                return Err(format!("VerylBridge has multiple clocks: found {n}"));
            }
        }
    }

    pub fn clock_count(&self) -> usize{
        self.clocks.len()
    }
    pub fn reset_default(&mut self) -> Result<(), String> {

        if self.resets.len() == 0 {
            return Ok(())
        };
        if self.resets.len() > 1 {
            return Err(format!("VerylBridge has multiple resets: found {}", self.resets.len()));
        }
        if self.clocks.len() == 0{
            return Err("VerylBridge has no clock".into());
        }
        if self.clocks.len() > 1 {
            return Err(format!("VerylBridge has multiple clocks"));
        }
        let reset_name = self.resets.keys().next().unwrap().clone();
        let clock_name = self.clocks.keys().next().unwrap().clone();
        self.reset(&reset_name, &clock_name)
    }

    pub fn reset(
        &mut self,
        reset_name: &str,
        clock_name: &str,
    ) -> Result<(), String>{
        let reset_info = self.port_info(reset_name)?.clone();

        if reset_info.type_kind != TypeKind::Reset {
            return Err(format!("Only TypeKind::Reset is supported, not '{:?}'", reset_info.type_kind));
        }
        let reset = self.resets.get(reset_name).ok_or_else(|| format!("VerylBridge has no reset: '{reset_name}'"))?;

        let clock = self.clocks.get(clock_name).ok_or_else(|| { format!("VerylBridge has no clock '{clock_name}'")})?;
        self.simulator.step_reset(clock, reset);

        Ok(())
    }

    pub fn read_value(
        &mut self,
        port: &str,
    ) -> Result<Value, String> {
        let port_info = self.port_info(port)?;

        if port_info.direction != Direction::Output{
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



    use crate::{ PortType, veryl_bridge::VerylBridge};

    use crate::veryl_loader::load_veryl_simulator;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name)
    }
        #[test]
    fn unsupported_reset_kind_are_rejected(){
        let file = fixture("unsupported_reset.veryl");

        let veryl_design = load_veryl_simulator(file.to_str().unwrap(), "UnsupportedReset").unwrap();

        let bridge = VerylBridge::new(
            veryl_design.simulator,
            veryl_design.design_info
        );

        assert!(bridge.is_err());
         
        
    }
    #[test]
    fn inout_port_is_rejected(){
        let file = fixture("in_out_port.veryl");
        assert!(load_veryl_simulator(file.to_str().unwrap(), "InOutPort").is_err());
    }
    #[test]
    fn writing_to_veryl_output_fails(){
        let file = fixture("test_bridge_register.veryl");

        let veryl_design = load_veryl_simulator(file.to_str().unwrap(), "TestRegister").unwrap();

        let mut bridge = VerylBridge::new(
            veryl_design.simulator,
            veryl_design.design_info
        ).unwrap();
        
        assert!(bridge.set_input("y", SignalValue::Data(10), PortType{width: 8, signed: false}).is_err())
    }
    #[test]
    fn reading_veryl_input_fails(){
        let file = fixture("test_bridge_register.veryl");

        let veryl_design = load_veryl_simulator(file.to_str().unwrap(), "TestRegister").unwrap();

        let mut bridge = VerylBridge::new(
            veryl_design.simulator,
            veryl_design.design_info
        ).unwrap();
        // "d" = input port
        assert!(bridge.read_value("d").is_err())
    }
    #[test]
    fn veryl_register_can_be_reset(){
        let file = fixture("test_bridge_register.veryl");

        let veryl_design = load_veryl_simulator(file.to_str().unwrap(), "TestRegister").unwrap();

        let mut bridge = VerylBridge::new(veryl_design.simulator, veryl_design.design_info).unwrap();

        bridge.set_input("d", SignalValue::Data(10), PortType { width: 8, signed: false }).unwrap();

        bridge.clock_default().unwrap();

        let value = bridge.read_value("y").unwrap();
        match value {
            Value::U64(v) => {
                assert_eq!(v.payload, 10)
            }
            _ => {
                panic!("Unsupported!")
            }
        };
        bridge.reset("rst", "clk").unwrap();

        let value = bridge.read_value("y").unwrap();
        match value {
            Value::U64(v) => {
                assert_eq!(v.payload, 0)
            }
            _ => {
                panic!("Unsupported!")
            }
        };

    }
    #[test]
    fn get_ir_info(){
        let file = fixture("test_bridge_multiple_clock.veryl");

        let veryl_design = load_veryl_simulator(file.to_str().unwrap(), "MultipleClock").unwrap();

        let bridge = VerylBridge::new(veryl_design.simulator, veryl_design.design_info).unwrap();
        
        for port in bridge.data_input_ports() {
            println!("Name: {}", port.name);
            println!("Kind: {:?}", port.type_kind);

            if port.type_kind.is_clock() {
                println!("Has clock");
            }
            if port.type_kind.is_reset(){
                println!("Has reset");
            }
        };
        // for port in bridge.output_ports() {
        //     println!("Veryl output: {:?}", port)
        // };

        
    }
    #[test]
    fn veryl_comb_to_seq(){
        let file = fixture("test_bridge_comb_seq.veryl");

        let veryl_design = load_veryl_simulator(file.to_str().unwrap(), "TestCombSeq").unwrap();

        let mut bridge = VerylBridge::new(veryl_design.simulator, veryl_design.design_info ).unwrap();


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

        bridge.clock("clk").unwrap();
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


        bridge.clock("clk").unwrap();
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

        let mut bridge = VerylBridge::new(veryl_design.simulator, veryl_design.design_info).unwrap();

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