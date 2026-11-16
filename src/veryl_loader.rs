use std::{fs, path::{Path, PathBuf}};

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
    source_path: &str,
    top: &str,
) -> Result<LoadVerylDesign, String> {

    let files = collect_veryl_files(Path::new(source_path))?;
    if files.is_empty() {
        return Err(format!("No Veryl files found in '{}'", source_path));
    }

    // clear veryl global symboltable
    symbol_table::clear();

    // create minimal projectmetadata
    let metadata= Metadata::create_default("prj").map_err(|e| format!("Cound not create Veryl metadata: {e:?}"))?;

    // analyze shared by the whole design
    let analyzer = Analyzer::new(&metadata);

    // parse all source files
    let mut parsers = Vec::new();

    for file in &files {
        let code = fs::read_to_string(file).map_err(|e| format!("Could not read Veryl file '{}': {e}", file.display()))?;

        let file_str = file.to_str().ok_or_else(|| format!("Invalid Veryl file path '{}'", file.display()))?;

        let parser = Parser::parse(&code, &file_str).map_err(|e| format!("Could not parse Veryl file '{}': {e}", file.display()))?;

        parsers.push(parser);
    }
    let mut errors = Vec::new();

    // Analyzer pass 1 for all files

    for parser in &parsers{
        errors.append(
            &mut analyzer.analyze_pass1(
                "prj", 
                &parser.veryl
            )
        );
    }
    // Run post_pass only after every source file has completed pass 1
    errors.append(
        &mut Analyzer::analyze_post_pass1()
    );

    // Analyzer pass 2 for all files

    let mut context = Context::default();
    let mut ir = analyzer_ir::Ir::default();

    for parser in &parsers {
        errors.append(
            &mut analyzer.analyze_pass2(
                &parser.veryl, 
                &mut context, 
                Some(&mut ir),
            )
        );
    }

    errors.append(
        &mut Analyzer::analyze_post_pass2(&ir)
    );

    if !errors.is_empty(){
        return Err(format!("Veryl analyzer errors: {errors:?}"));
    }

    // Build simulator IR from selected top module

    let config = Config::default();

    let sim_ir = build_ir(&ir, top.into(), &config).map_err(|e| format!("Could not build Veryl simulator IR: '{e:?}'"))?;
    let simulator = Simulator::new(sim_ir, None);
    // Only extract ports from selected top module
    let design_info = extract_design_info(top.into(), &simulator)?;
    Ok(LoadVerylDesign { simulator, design_info })

}

fn collect_veryl_files(path: &Path) -> Result<Vec<PathBuf>, String> {
    if path.is_file() {
        if path.extension().and_then(|x| x.to_str()) == Some("veryl") {
            return Ok(vec![path.to_path_buf()]);
        }
        return Err(format!("'{}' in not a Veryl file", path.display()));
    }

    if path.is_dir(){
        let mut files = Vec::new();

        for entry in fs::read_dir(path).map_err(|e| format!("Could not read directory: '{}': {e}", path.display()))?{
            let entry = entry.map_err(|e| format!("Could not read directory entry: {e}"))?;

            let entry_path = entry.path();

            if entry_path.is_dir(){
                files.extend(collect_veryl_files(&entry_path)?);
            }else if entry_path.extension().and_then(|x| x.to_str()) == Some("veryl"){
                files.push(entry_path);
            }
        }
        files.sort();

        return Ok(files);
    }
    Err(format!("Veryl source path '{}' does not exist", path.display()))

}