use clvmr::chia_dialect::ChiaDialect;
use clvmr::node::Node;
use clvmr::run_program::run_program_with_test_samples_option;
use clvmr::serialize::node_to_bytes;
use clvmr::{allocator::Allocator, serialize::node_from_bytes};
use serde_json::{json, Value};
use std::{env, fs};

fn main() {
    let cli_args: Vec<String> = env::args().collect();

    if cli_args.len() != 3 {
        println!("Hex-encoded serialized program and args are not specified");
        return;
    }

    let program_bytes = fs::read(&cli_args[1]).expect("Failed to read program");
    let args_bytes = fs::read(&cli_args[2]).expect("Failed to read args");

    let mut a = Allocator::new();

    let program = node_from_bytes(
        &mut a,
        hex::decode(&program_bytes.as_slice()).unwrap().as_slice(),
    )
    .unwrap();
    let args = node_from_bytes(
        &mut a,
        hex::decode(&args_bytes.as_slice()).unwrap().as_slice(),
    )
    .unwrap();

    let d = ChiaDialect::new(0);

    let (res, test_samples) = run_program_with_test_samples_option(
        &mut a,
        &d,
        program,
        args,
        18446744073709551615,
        None,
        true,
    );

    let ret = res.unwrap();

    println!("Cost: {}", ret.0);

    let mut samples: Vec<Value> = Vec::new();

    for x in test_samples.iter() {
        samples.push(json!({
            "op_code": hex::encode(node_to_bytes(&Node::new(&a, x.op_code)).unwrap().as_slice()),
            "args": hex::encode(node_to_bytes(&Node::new(&a, x.args)).unwrap().as_slice()),
            "cost": x.cost,
            "ret": hex::encode(node_to_bytes(&Node::new(&a, x.ret)).unwrap().as_slice()),
        }));
    }

    let ret_path = "./ret.clvm";

    println!("Writing results (file {})", ret_path);

    fs::write(ret_path, node_to_bytes(&Node::new(&a, ret.1)).unwrap()).unwrap();

    let tests_path = "./tests.json";

    println!("Writing tests (file {})", tests_path);

    fs::write(tests_path, serde_json::to_string(&samples).unwrap()).unwrap();

    println!("Done.")
}
