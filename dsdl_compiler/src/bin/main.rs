extern crate getopts;
#[macro_use]
extern crate log;
extern crate badlog;
extern crate dsdl_parser;
extern crate dsdl_compiler;
#[macro_use]
extern crate quote;

mod opts;

use std::fs::File;
use std::io::Write;

use std::str::FromStr;

use opts::InputFlags;

use dsdl_parser::DSDL;

use dsdl_compiler::Compile;
use dsdl_compiler::CompileConfig;

use dsdl_compiler::config::*;

fn main() {
    badlog::init(Some("info"));
    
    let flags = InputFlags::read();
    
    if flags.help {
        opts::print_usage();
        return;
    }

    if flags.version {
        opts::print_version();
        return;
    }

    let input = if let Some(path) = flags.input.clone() {
        path
    } else {
        opts::print_usage();
        println!("\nInput needs to be specified");
        return;
    };

    let output = if let Some(path) = flags.output.clone() {
        path
    } else {
        opts::print_usage();
        println!("\nOutput needs to be specified");
        return;
    };

    let dsdl = match DSDL::read(input) {
        Ok(dsdl) => dsdl,
        Err(error) => {
            error!("errored when reading DSDL: {}", error);
            return;
        },
    };

    let mut compile_config = CompileConfig::default();
    compile_config.data_type_signature = flags.data_type_signature;
    compile_config.derive_default = if let Some(s) = flags.derive_default {
        if let Ok(derive_default) = DeriveDefault::from_str(&s) {
            derive_default
        } else {
            error!("Error reading setting `derive-default`");
            opts::print_usage();
            return;
        }
    } else {
        DeriveDefault::default()
    };

    let items = dsdl.compile(&compile_config);
    
    let mut file = match File::create(output) {
        Ok(file) => file,
        Err(error) => {
            error!("errored when creating output file: {}", error);
            return;
        },
    };


    let tokens = quote!{#(#items)*};
    
    match file.write_all(tokens.as_str().as_bytes()) {
        Ok(_) => (),
        Err(error) => error!("errored when writing to output file: {}", error),
    }
    
}
