use std::{str::FromStr};

use veryl_simulator::Simulator;

use veryl_analyzer::{
    ir::{VarKind::Output, VarPath}, symbol::{Direction, SymbolKind, TypeKind}, symbol_table,
};

use crate::PortType;

#[derive(Debug, Clone)]
pub struct VerylPortInfo {
    pub name: String,
    pub port_type: PortType,
    pub direction: Direction,
    pub type_kind: TypeKind,
}

#[derive(Debug, Clone)]
pub struct VerylDesignInfo {
    pub ports: Vec<VerylPortInfo>,
}

pub fn extract_design_info(
    top_module: &str,
    simulator: &Simulator,
) ->Result<VerylDesignInfo, String>{
    let symbol = symbol_table::get_all();

    let module = symbol.into_iter().find(|symbol| {
        let name = format!("{}", symbol.token);

        name == top_module && matches!(symbol.kind, SymbolKind::Module(_))
        })
        .ok_or_else(|| {
            format!("Could not find Veryl module '{top_module}'")
    })?;

    let module_property = match module.kind {
        SymbolKind::Module(property) => property,
        _ => unreachable!(),
    };

    let mut ports = Vec::new();

    for port in module_property.ports {
        let name = format!("{}", port.name());
        let property = port.property();

        let direction= match property.direction {
            Direction::Input => Direction::Input,
            Direction::Output => Direction::Output,
            _=> {
                return Err(format!("Direction not currently supported: {:?}", property.direction));
            }
        };
        
        let port_type = port_type_from_ir(
            simulator,
            &name,
        )?;

        let type_kind = property.r#type.kind.clone();
        ports.push(VerylPortInfo {
            name,
            port_type,
            direction,
            type_kind,
        })
    }
    Ok(VerylDesignInfo { ports })
}

fn port_type_from_ir(
    simulator: &Simulator,
    port: &str,
) -> Result<PortType, String>{
    let path = VarPath::from_str(port).map_err(|_| {format!("Invalid Veryl port path '{port}'")})?;

    let id = simulator.ir.ports.get(&path).ok_or_else(|| format!("Veryl port '{port}' was not found"))?;

    let variable = simulator.ir.module_variables.variables.get(id).ok_or_else(|| {format!("Could not find variable metadata for Veryl port '{port}'")})?;

    let width = u8::try_from(variable.width).map_err(|_| {format!("Port '{}' has unsupported width {}", port, variable.width)})?;

    if width == 0 || width > 32 {
        return Err(format!("Port: {} has unsupported width: {}", port, width));
    }
    Ok(PortType { width, signed: variable.r#type.signed })
}

