use std::fs;

use veryl_analyzer::{
    Analyzer,
    Context,
    ir as analyzer_ir,
    symbol_table,
};

use veryl_metadata::Metadata;
use veryl_parser::Parser;

use veryl_simulator::{
    Config,
    Simulator,
    ir::build_ir,
};

use crate::{VerylDesignInfo, extract_design_info};

pub struct LoadVerylDesign{
    pub simulator: Simulator,
    pub design_info: VerylDesignInfo,
}

pub fn load_veryl_simulator(
    file: &str,
    top: &str,
) -> Result<LoadVerylDesign, String> {
    let code = fs::read_to_string(file).expect("Could not read Veryl file");

    // clear veryl global symboltable
    symbol_table::clear();

    // create minimal projectmetadata
    let metadata= Metadata::create_default("prj").unwrap();

    // parser representation / syntax tree
    let parser = Parser::parse(&code, &file).unwrap();

    // analyze code
    let analyzer = Analyzer::new(&metadata);

    
    let mut context = Context::default();
    let mut ir = analyzer_ir::Ir::default();

    let mut errors = vec![];

    errors.append(
        &mut analyzer.analyze_pass1(
            "prj",
            &parser.veryl,
        )
    );

    errors.append(
        &mut Analyzer::analyze_post_pass1()
    );

    errors.append(
        &mut analyzer.analyze_pass2(
            &parser.veryl,
            &mut context,
            Some(&mut ir),
        )
    );

    errors.append(
        &mut Analyzer::analyze_post_pass2(&ir)
    );

    assert!(
        errors.is_empty(),
        "Veryl analyzer errors: {errors:?}"
    );

    let config = Config::default();

    let sim_ir = build_ir(&ir, top.into(), &config).unwrap();
    let simulator = Simulator::new(sim_ir, None);
    let design_info = extract_design_info(top.into(), &simulator)?;
    Ok(LoadVerylDesign { simulator, design_info })

}