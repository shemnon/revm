use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

use bytes::Bytes;
use primitive_types::{H160, H256};
use ruint::aliases::U256;
use sha3::{Digest, Keccak256};
use structopt::StructOpt;

use revm::{Bytecode, Env, ExecutionResult, TransactTo};
// use rustc_hex::ToHex;
// 
// use super::super::statetest::CustomPrintTracer;

use crate::runner::runner::RunError;

#[derive(StructOpt, Debug)]
pub struct Cmd {
    #[structopt(long, parse(try_from_str = parse_hex))]
    code: Bytes,
    #[structopt(long, default_value = "10000000")]
    gas: u64,
    #[structopt(long, parse(try_from_str = parse_hex), default_value = "")]
    input: Bytes,
    // #[structopt(long, parse(from_os_str))]
    // alloc: Option<PathBuf>,
    #[structopt(long, parse(try_from_str = parse_h160))]
    receiver: Option<H160>,
    #[structopt(long, parse(try_from_str = parse_h160))]
    sender: Option<H160>,
    #[structopt(long)]
    value: Option<U256>,
}

fn parse_hex(src: &str) -> Result<Bytes, hex::FromHexError> {
    Ok(Bytes::from(hex::decode(src)?))
}

pub fn parse_h160(input: &str) -> Result<H160, <H160 as FromStr>::Err> {
    H160::from_str(input)
}


impl Cmd {
    pub fn run(&self) -> Result<(), RunError> {
        // for path in &self.path {
        //     println!("Start running tests on: {path:?}");
        //     run(test_files)?
        // }
        let env = self.create_env()?;
        //warmup
        self.run_script(&env)?;
        let mut results = [self.run_script(&env)?, self.run_script(&env)?,self.run_script(&env)?];
        results.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("- gps {:.3}", results[1]);
        Ok(())
    }

    fn create_env(&self) -> Result<Env, RunError> {
        let mut env: Env = Env::default();

        env.block.gas_limit = U256::from_limbs([0, 0, 0, self.gas]);
        env.block.number = U256::from_str_radix("1", 10).unwrap();
        env.block.coinbase = H160::repeat_byte(0);
        env.block.timestamp = U256::ZERO;
        env.block.difficulty = U256::ZERO;
        env.block.basefee = U256::ZERO;

        env.tx.caller = self.sender.unwrap_or_else(|| H160::repeat_byte(0));
        env.tx.gas_limit = self.gas;
        env.tx.value = self.value.unwrap_or_else(|| U256::ZERO);
        env.tx.data = self.input.clone();
        env.tx.gas_priority_fee = None;
        env.tx.chain_id = Option::from(1);
        env.tx.nonce = Option::from(1);

        env.tx.transact_to = if let Some(to) = self.receiver {
            TransactTo::Call(to)
        } else {
            TransactTo::create()
        };

        Ok(env)
    }

    fn run_script(&self, env: &Env) -> Result<f64, RunError> {
        let mut database = revm::InMemoryDB::default();
        let acc_info = revm::AccountInfo {
            balance: U256::from_limbs([2,2,2,2]),
            code_hash: H256::from_slice(Keccak256::digest(&self.code).as_slice()), //try with dummy hash.
            code: Some(Bytecode::new_raw(self.code.clone())),
            nonce: 1, //FIXME
        };
        database.insert_account_info(self.receiver.unwrap(), acc_info);
        // insert storage:
        // for (&slot, &value) in info.storage.iter() {
        //     let _ = database.insert_account_storage(*address, slot, value);
        // }


        let mut database_cloned = database.clone();
        let mut evm = revm::new();
        evm.database(&mut database_cloned);
        evm.env = env.clone();
        // do the deed

        let timer = Instant::now();
        let ExecutionResult {
            exit_reason,
            gas_used,
            gas_refunded,
            logs,
            ..
        // } = evm.inspect_commit(CustomPrintTracer::new());
        } = evm.transact_commit();
        let timer = timer.elapsed();

        let gps = gas_used as f64 * 1_000.0 / timer.as_nanos() as f64;
        // println!("\
        // exit reason {exit_reason:?}\n\
        // gas used {gas_used}\n\
        // gas refunded {gas_refunded}\n\
        // logs {logs:?}\n");
        // println!("Test Mgps {:.3}", gps);


        Ok(gps)
    }
}