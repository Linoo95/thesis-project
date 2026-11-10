use syncrim::signal::{SignalValue};

use veryl_simulator::ir::Value;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortType {
    pub width: u8, // 1..32
    pub signed: bool,
}

pub fn convert_syncrim_to_veryl(
    value: SignalValue,
    source_type: PortType,
    destination_type: PortType,
) -> Result<Value, String> {

    check_port_type(source_type, destination_type)?;

    let veryl_value = match value {
        SignalValue::Data(data) => {
            let payload = extend_value(data, source_type, destination_type)?;
            Value::new(payload as u64, destination_type.width as usize, destination_type.signed)
        },
        SignalValue::Unknown => {
            Value::new_x(destination_type.width as usize, destination_type.signed)
        },
        SignalValue::DontCare => {
            Value::new_x(destination_type.width as usize, destination_type.signed)
        },
        SignalValue::Uninitialized => {
            Value::new_x(destination_type.width as usize, destination_type.signed)
        },
    };

    Ok(veryl_value)
}

fn fits_in_width(value: u32, width: u8) -> bool {
    if width == 0 || width > 32 {
        return false
    }
    if width == 32 {
        return true;
    }
    value < (1u32 << width)
}

fn extend_value(
    data: u32,
    source_type: PortType,
    destination_type: PortType,
) -> Result<u32, String>{

    if !fits_in_width(data, source_type.width){
        return Err(format!("Value {} does not fit in {} bits", data, source_type.width));
    }

    if source_type.width == destination_type.width {
        return Ok(data);
    }

    // Unsigned zero extend automatically
    // data is already in a u32
    if !source_type.signed {
        return Ok(data);
    }

    let sign_bit = 1u32 << (source_type.width - 1);

    // positive sign value -> zero extend
    if data & sign_bit == 0 {
        return Ok(data);
    }

    let source_mask = if source_type.width == 32 {
        u32::MAX
    }else {
        (1u32 << source_type.width) - 1
    };

    let destination_mask = if destination_type.width == 32 {
        u32::MAX
    } else {
        (1u32 << destination_type.width) - 1
    };

    let extend = (data & source_mask) | (destination_mask & !source_mask);

    Ok(extend)

}


fn check_port_type(source_type: PortType, destination_type: PortType) -> Result<(), String>{
    if source_type.width == 0 || source_type.width > 32{
        return Err(format!("Unsupported Veryl source width: {}", source_type.width));
    }
    if destination_type.width == 0 || destination_type.width > 32 {
        return  Err(format!("Unsupported SyncRim destiantion width: {}", destination_type.width));
    }
    if source_type.width > destination_type.width {
        return Err(format!("Need truncation to convert {}-bit Veryl to {}-bit SyncRim", source_type.width, destination_type.width));
    }
    if source_type.signed != destination_type.signed {
        return Err(format!("Signedness mismatch: source signed={}, destination={}", source_type.signed, destination_type.signed));
    }
    Ok(())
}

pub fn convert_veryl_to_syncrim(
    value: Value,
    source_type: PortType,
    destination_type: PortType,
) -> Result<SignalValue, String> {
    check_port_type(source_type, destination_type)?;

    let value = match value {
        Value::U64(v) => {
            v
        },
        Value::BigUint(_) => {
            return  Err("Veryl BigUint cannot be represented by SyncRim".into());
        }, 
    };
    let mask = (1u64 << value.width) - 1;
    // check so there are no bits outside of width
    if value.payload & !mask !=0 {
        return Err(format!("Veryl payload {} does not fit in width {}", value.payload, value.width));
    }

    let payload = value.payload & mask;
    let mask_xz = value.mask_xz & mask;
    if mask_xz != 0 {
        if mask_xz == mask && payload == 0 {
            return Ok(SignalValue::Unknown)
        }else {
            return Err("Veryl value contains partial x or z that SyncRim cannot represent".into());
        }
    }
    let data = u32::try_from(payload).map_err(|_| "Veryl value does not fit in u32")?;
    
    let extend = extend_value(data, source_type, destination_type)?;

    Ok(SignalValue::Data(extend))

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn different_width_test(){
        for width in 1..32 {
            let source = PortType {
                width: width,
                signed: false,
            };
            let destination = PortType {
                width: width,
                signed: false,
            };
            let data = SignalValue::Data(1);
            let result_1 = convert_syncrim_to_veryl(data, source, destination).unwrap();
            
            match result_1 {
                Value::U64(v) => {
                    assert_eq!(v.payload, 0b1);
                    assert_eq!(v.width, width as u32);
                    assert_eq!(v.signed, false);
                    assert_eq!(v.mask_xz, 0);
                }
                _ => {
                    panic!("Expected Value::U64");
                }   
            }
            let value = Value::new(0b1, width as usize, false);
            let signal_value = convert_veryl_to_syncrim(value, source, destination).unwrap();

            assert_eq!(signal_value, SignalValue::Data(0b1));
        }
    }

    #[test]
    fn invalid_width_source(){
        let source = PortType {
            width: 33,
            signed: false,
        };
        let destination = PortType {
            width: 8,
            signed: false,
        };

        let data = SignalValue::Data(1);
        assert!(convert_syncrim_to_veryl(data, source, destination).is_err());
        let value = Value::new(0b1, 8, false);
        assert!(convert_veryl_to_syncrim(value, source, destination).is_err());

        let source = PortType {
            width: 0,
            signed: false,
        };

        let data = SignalValue::Data(1);
        assert!(convert_syncrim_to_veryl(data, source, destination).is_err());
        let value = Value::new(0b1, 8, false);
        assert!(convert_veryl_to_syncrim(value, source, destination).is_err());
    }

    #[test]
    fn invalid_width_destination(){
        let source = PortType {
            width: 8,
            signed: false,
        };
        let destination = PortType {
            width: 33,
            signed: false,
        };

        let data = SignalValue::Data(1);
        assert!(convert_syncrim_to_veryl(data, source, destination).is_err());
        let value = Value::new(0b1, 8, false);
        assert!(convert_veryl_to_syncrim(value, source, destination).is_err());

        let destination = PortType {
            width: 0,
            signed: false,
        };
        let data = SignalValue::Data(1);
        assert!(convert_syncrim_to_veryl(data, source, destination).is_err());
        let value = Value::new(0b1, 8, false);
        assert!(convert_veryl_to_syncrim(value, source, destination).is_err());
    }
    #[test]
    fn value_bigger_then_width(){
        let source = PortType {
            width: 8,
            signed: false,
        };
        let destination = PortType {
            width: 8,
            signed: false,
        };
        let data = SignalValue::Data(256);
        assert!(convert_syncrim_to_veryl(data, source, destination).is_err());
        let value = Value::new(256, 8, false);
        assert!(convert_veryl_to_syncrim(value, source, destination).is_err());
    }
    #[test]
    fn invalid_narrowing(){
        let source = PortType {
            width: 12,
            signed: false,
        };
        let destination = PortType {
            width: 8,
            signed: false,
        };
        let data = SignalValue::Data(1);
        assert!(convert_syncrim_to_veryl(data, source, destination).is_err());
        let value = Value::new(0b1, 12, false);
        assert!(convert_veryl_to_syncrim(value, source, destination).is_err())
    }
    #[test]
    fn unsigned_widening(){
        let source = PortType {
            width: 8,
            signed: false,
        };
        let destination = PortType {
            width: 12,
            signed: false,
        };
        let data = SignalValue::Data(0b1111_1111);
        let result_1 = convert_syncrim_to_veryl(data, source, destination).unwrap();
        
        match result_1 {
            Value::U64(v) => {
                assert_eq!(v.payload, 0b0000_1111_1111);
                assert_eq!(v.width, 12);
                assert_eq!(v.signed, false);
                assert_eq!(v.mask_xz, 0);
            }
            _ => {
                panic!("Expected Value::U64");
            }   
        }
        let value = Value::new(0b1111_1111, 8, false);
        let signal_value = convert_veryl_to_syncrim(value, source, destination).unwrap();

        assert_eq!(signal_value, SignalValue::Data(0b0000_1111_1111));
    }
    #[test]
    fn signed_same_width(){
        let source = PortType {
            width: 8,
            signed: true,
        };
        let destination = PortType {
            width: 8,
            signed: true,
        };

        let data = SignalValue::Data(0b1111_1111);
        let result_1 = convert_syncrim_to_veryl(data, source, destination).unwrap();
        
        match result_1 {
            Value::U64(v) => {
                assert_eq!(v.payload, 0b1111_1111);
                assert_eq!(v.width, 8);
                assert_eq!(v.signed, true);
                assert_eq!(v.mask_xz, 0);
            }
            _ => {
                panic!("Expected Value::U64");
            }   
        }
        let value = Value::new(0b1111_1111, 8, true);
        let signal_value = convert_veryl_to_syncrim(value, source, destination).unwrap();

        assert_eq!(signal_value, SignalValue::Data(0b1111_1111));

    }
    #[test]
    fn signed_negative_widening(){
        let source = PortType {
            width: 8,
            signed: true,
        };
        let destination = PortType {
            width: 12,
            signed: true,
        };
        let data = SignalValue::Data(0b1111_1111);
        let result_1 = convert_syncrim_to_veryl(data, source, destination).unwrap();
        
        match result_1 {
            Value::U64(v) => {
                assert_eq!(v.payload, 0b1111_1111_1111);
                assert_eq!(v.width, 12);
                assert_eq!(v.signed, true);
                assert_eq!(v.mask_xz, 0);

            }
            _ => {
                panic!("Expected Value:U64")
            }
        }
        let value = Value::new(0b1111_1111, 8, true);

        let signal_value = convert_veryl_to_syncrim(value, source, destination).unwrap();

        assert_eq!(signal_value, SignalValue::Data(0b1111_1111_1111));
    }
    #[test]
    fn signed_positive_widening(){
        let source = PortType {
            width: 8,
            signed: true,
        };
        let destination = PortType {
            width: 12,
            signed: true,
        };
        let data = SignalValue::Data(0b0000_1111);
        let result_1 = convert_syncrim_to_veryl(data, source, destination).unwrap();
        
        match result_1 {
            Value::U64(v) => {
                assert_eq!(v.payload, 0b0000_0000_1111);
                assert_eq!(v.width, 12);
                assert_eq!(v.signed, true);
                assert_eq!(v.mask_xz, 0);
            },
            _ => {
                panic!("Expected Value::U64")
            }

        }
        let value = Value::new(0b0000_1111, 8, true);
        let signal_value = convert_veryl_to_syncrim(value, source, destination).unwrap();

        assert_eq!(signal_value, SignalValue::Data(0b0000_0000_1111));
        
    }

    #[test]
    fn unknown_value(){
        let source = PortType {
            width: 8,
            signed: false,
        };
        let destination = PortType {
            width: 12,
            signed: false,
        };
        let data = SignalValue::Unknown;
        let result_1 = convert_syncrim_to_veryl(data, source, destination).unwrap();
        
        match result_1 {
            Value::U64(v) => {
                assert_eq!(v.payload, 0b0000_0000_0000);
                assert_eq!(v.width, 12);
                assert_eq!(v.signed, false);
                assert_eq!(v.mask_xz, 0b1111_1111_1111);
            },
            _ => {
                panic!("Expected Value::U64")
            }

        }
        let value = Value::from_u128(0b0000_0000, 0b1111_1111, 8, false);
        let signal_value = convert_veryl_to_syncrim(value, source, destination).unwrap();

        assert_eq!(signal_value, SignalValue::Unknown);
    }

    #[test]
    fn unknown_roundtrip(){
        let source = PortType {
            width: 8,
            signed: false,
        };
        let destination = PortType {
            width: 8,
            signed: false,
        };
        let data = SignalValue::Unknown;
        let result_1 = convert_syncrim_to_veryl(data, source, destination).unwrap();
        let signal_value = convert_veryl_to_syncrim(result_1, source, destination).unwrap();

        assert_eq!(signal_value, SignalValue::Unknown)
    }
    #[test]
    fn twelve_bit_max_value(){
        let port = PortType {
            width: 12,
            signed: false,
        };

        assert!(convert_syncrim_to_veryl(SignalValue::Data(4095), port, port).is_ok());

        assert!(convert_syncrim_to_veryl(SignalValue::Data(4096), port, port).is_err());
    }

    #[test]
    fn thirty_two_bit_max_value(){
        let port = PortType {
            width: 32,
            signed: false,
        };

        assert!(convert_syncrim_to_veryl(SignalValue::Data(u32::MAX), port, port).is_ok());
    }

    #[test]
    fn veryl_partial_x_to_syncrim_cannot_be_represented_exactly(){
        let source = PortType {
            width: 8,
            signed: false,
        };
        let destination = PortType {
            width: 12,
            signed: false,
        };
        let value = Value::from_u128(0b0000_0001, 0b0000_0010, 8, false);

        let result = convert_veryl_to_syncrim(value, source, destination);
        
        assert!(result.is_err());
    }


    #[test]
    fn syncrim_u8_round_trip_preserves_value(){

        let source = PortType {
            width: 8,
            signed: false,
        };
        let destination = PortType {
            width: 8,
            signed: false,
        };
        let signal_value = SignalValue::Data(5);

        let veryl_result = convert_syncrim_to_veryl(signal_value, source, destination).unwrap();

        let sync_value = convert_veryl_to_syncrim(veryl_result, source, destination).unwrap();

        assert_eq!(sync_value, SignalValue::Data(5));
    }

    #[test]
    fn veryl_u8_round_trip_preserves_value(){
        let source = PortType {
            width: 8,
            signed: false,
        };
        let destination = PortType {
            width: 8,
            signed: false,
        };
        let value = Value::new(5, 8, false);

        let sync_result = convert_veryl_to_syncrim(value, source, destination).unwrap();

        let veryl_value = convert_syncrim_to_veryl(sync_result, source, destination).unwrap();

        match veryl_value {
            Value::U64(v) => {
                assert_eq!(v.payload, 5);
                assert_eq!(v.mask_xz, 0);
                assert_eq!(v.signed, false);
                assert_eq!(v.width, 8);

            }
            Value::BigUint(_) => {
                panic!("Expected::U64");
            } 
        }  
    }

    #[test]
    fn veryl_all_z_to_syncrim_fails(){
        let source = PortType {
            width: 8,
            signed: false,
        };
        let destination = PortType {
            width: 8,
            signed: false,
        };
        let value = Value::from_u128(0b1111_1111, 0b1111_1111, 8, false);

        assert!(convert_veryl_to_syncrim(value, source, destination).is_err())
    }

    #[test]
    fn syncrim_dontcare_to_veryl_all_x(){
        let source = PortType {
            width: 8,
            signed: false,
        };
        let destination = PortType {
            width: 8,
            signed: false,
        };
        let signal= SignalValue::DontCare;
        
        let result = convert_syncrim_to_veryl(signal, source, destination).unwrap();
        match result {
            Value::U64(v) => {
                assert_eq!(v.payload, 0);
                assert_eq!(v.width, 8);
                assert_eq!(v.mask_xz, 0b1111_1111);
            }
            _ => panic!("Expected U64")
        }
    }

    #[test]
    fn syncrim_uninitialized_to_veryl_all_x(){
        let source = PortType {
            width: 8,
            signed: false,
        };
        let destination = PortType {
            width: 8,
            signed: false,
        };
        let signal = SignalValue::Uninitialized;
        

        let result = convert_syncrim_to_veryl(signal, source, destination).unwrap();
        match result {
            Value::U64(v) => {
                assert_eq!(v.payload, 0);
                assert_eq!(v.width, 8);
                assert_eq!(v.mask_xz, 255);
            }
            _ => panic!("Expected U64")
        }
    }

    #[test]
    fn syncrim_unknown_round_trip(){
        let source = PortType {
            width: 8,
            signed: false,
        };
        let destination = PortType {
            width: 8,
            signed: false,
        };

        let signal = SignalValue::Unknown;

        let veryl_result = convert_syncrim_to_veryl(signal, source, destination).unwrap();

        let result = convert_veryl_to_syncrim(veryl_result, source, destination).unwrap();

        assert_eq!(result, SignalValue::Unknown);
    }

    #[test]
    fn veryl_biguint_to_syncrim_fails(){
        let source = PortType {
            width: 8,
            signed: false,
        };
        let destination = PortType {
            width: 8,
            signed: false,
        };
        let value = Value::new(5, 65, false);
        assert!(convert_veryl_to_syncrim(value, source, destination).is_err())
    }


    #[test]
    fn veryl_partial_z_to_syncrim_fails(){
        let source = PortType {
            width: 8,
            signed: false,
        };
        let destination = PortType {
            width: 8,
            signed: false,
        };
        let value = Value::from_u128(0b0001_0010, 0b0001_0010, 8, false);

        assert!(convert_veryl_to_syncrim(value, source, destination).is_err());
    }

    #[test]
    fn veryl_value_too_large_for_u8_fails(){
        let source = PortType {
            width: 8,
            signed: false,
        };
        let destination = PortType {
            width: 8,
            signed: false,
        };

        let value = Value::new(256, 8, false);

        assert!(convert_veryl_to_syncrim(value, source, destination).is_err());
    }
}