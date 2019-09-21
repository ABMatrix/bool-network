use canonical_serialization::{SimpleDeserializer, SimpleSerializer};
use clap::{App, Arg, ArgMatches, SubCommand};
use compiler;
use core::convert::TryFrom;
use crypto::{signing, signing::KeyPair, PrivateKey, PublicKey};
use hex;
use mock::{account::Account, common, compile::*, gas_costs};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    str,
};
use vm::{
    bytecode_verifier::VerifiedModule,
    def::file_format::CompiledModule,
    types::{
        account_config::AccountResource,
        transaction::{Program, SignedTransaction, TransactionArgument},
        AccessPath, AccountAddress,
    },
};
// use num_traits::real::Real;
use dirs;

fn main() {
    let args = App::new("bool node move cli")
        .version("0.3.0")
        .author("Jason")
        .about("For making move transaction and assist。")
        .subcommand(generate_sub_command_tx())
        .subcommand(generate_sub_command_get_access_path())
        .subcommand(generate_sub_command_decode())
        .subcommand(generate_sub_command_get_address())
        .subcommand(generate_sub_command_get_public_key())
        .subcommand(generate_sub_command_account())
        .get_matches();

    if let (command, Some(matches)) = args.subcommand() {
        match command {
            "tx" => deal_command_make_tx(matches),
            "account" => deal_command_account(matches),
            "decode" => deal_command_decode(matches),
            "get_access_path" => deal_command_get_access_path(matches),
            "get_address" => deal_command_get_address(matches),
            "get_public_key" => deal_command_get_public_key(matches),
            _ => unimplemented!(),
        }
    }
}

fn generate_sub_command_tx<'a, 'b>() -> App<'a, 'b> {
    let subcommand = SubCommand::with_name("tx")
        .about("make libra transaction")
        .arg(
            Arg::with_name("faucet_account_file")
                .short("f")
                .long("faucet_account_file")
                .takes_value(true)
                .help("faucet account file path."),
        )
        .arg(
            Arg::with_name("key")
                .short("k")
                .long("key")
                .takes_value(true)
                .help("sender's private key. invalid if faucet account file present."),
        )
        .arg(
            Arg::with_name("program")
                .short("m")
                .long("program")
                .takes_value(true)
                .help("program should one of `create_account`, `mint` or `transfer`."),
        )
        .arg(
            Arg::with_name("recipient")
                .short("r")
                .long("recipient")
                .takes_value(true)
                .help("pubkey of recipient."),
        )
        .arg(
            Arg::with_name("value")
                .short("v")
                .long("value")
                .takes_value(true)
                .help("number of coins."),
        )
        .arg(
            Arg::with_name("sequence_number")
                .short("s")
                .long("sequence_number")
                .takes_value(true)
                .help("sender account sequence number."),
        )
        .arg(
            Arg::with_name("compiled_file")
                .short("cf")
                .long("compiled_file")
                .takes_value(true)
                .help("complied file path."),
        )
        .arg(
            Arg::with_name("params")
                .short("p")
                .long("params")
                .takes_value(true)
                .help("script parameters"),
        );
    subcommand
}

fn parse_pubkey_coin(args: &ArgMatches) -> (PublicKey, u64) {
    let receiver = args
        .value_of("recipient")
        .map(|input| hex::decode(&input[2..]))
        .map(|data| PublicKey::from_slice(&data.unwrap()))
        .expect("parse recipient public key error")
        .unwrap();

    let num_coins = args
        .value_of("value")
        .expect("should provide number of coins")
        .parse()
        .unwrap();
    (receiver, num_coins)
}

fn parse_address_coin(args: &ArgMatches) -> (AccountAddress, u64) {
    let receiver = args
        .value_of("recipient")
        .map(|input| AccountAddress::from_hex_literal(input))
        .expect("parse recipient address error")
        .unwrap();

    let num_coins = args
        .value_of("value")
        .expect("should provide number of coins")
        .parse()
        .unwrap();
    (receiver, num_coins)
}

fn deal_command_make_tx(args: &ArgMatches) {
    let key_pair = args
        .value_of("key")
        .map(|input| hex::decode(&input[2..]))
        .map(|data| PrivateKey::from_slice(&data.unwrap()))
        .map(|key| KeyPair::new(key.unwrap()))
        .expect("should private key");

    let sequence_number = args
        .value_of("sequence_number")
        .expect("should sequence number")
        .parse()
        .unwrap();

    let signed_txn = match args
        .value_of("program")
        .expect("should provide program method")
    {
        "create_account" => {
            let (receiver, num_coins) = parse_address_coin(&args);
            common::create_account_txn(
                &Account::from_keypair(key_pair),
                &Account::mock_from_address(receiver),
                sequence_number,
                num_coins,
            )
        }
        "mint" => {
            let (receiver, num_coins) = parse_address_coin(&args);
            common::mint_txn(
                &Account::from_keypair(key_pair),
                &Account::mock_from_address(receiver),
                sequence_number,
                num_coins,
            )
        }
        "transfer" => {
            let (receiver, num_coins) = parse_address_coin(&args);
            common::peer_to_peer_txn(
                &Account::from_keypair(key_pair),
                &Account::mock_from_address(receiver),
                sequence_number,
                num_coins,
            )
        }
        "publish" => {
            let params = args
                .value_of("params")
                .map_or(vec![], |p| parse_script_args(p).expect("invalid params"));
            compile_and_publish(
                &Account::from_keypair(key_pair),
                args.value_of("compiled_file").expect("should has file"),
                params,
                sequence_number,
            )
        }
        _ => unimplemented!(),
    };

    let se_txn = SimpleSerializer::<Vec<u8>>::serialize(&signed_txn).unwrap();
    let hex = hex::encode(se_txn);
    print!("0x{}", hex);
}

const APP_DIR: &str = "Caster";
const MODULE_DIR: &str = "modules";

fn home_path() -> PathBuf {
    let root = dirs::data_local_dir().expect("should has local directory");
    root.join(APP_DIR).join(MODULE_DIR)
}

fn parse_script_args(data: &str) -> Result<Vec<TransactionArgument>, &str> {
    Ok(vec![])
}

fn load_local_modules(home: &PathBuf) -> Vec<VerifiedModule> {
    let mut modules = vec![];
    if home.exists() {
        for entry in home.read_dir().expect("read_dir call failed") {
            if let Ok(entry) = entry {
                let data = fs::read(entry.path()).unwrap();
                if let Ok(module) = CompiledModule::deserialize(&data) {
                    if let Ok(v_module) = VerifiedModule::new(module) {
                        modules.push(v_module);
                    } else {
                        println!("unverified module! {:?}", entry.path());
                    }
                } else {
                    println!("unkonwn module! {:?}", entry.path());
                }
            }
        }
    }
    modules
}

fn compile_and_publish(
    sender: &Account,
    path: &str,
    args: Vec<TransactionArgument>,
    seq_num: u64,
) -> SignedTransaction {
    let path = Path::new(path);
    let home_dir = home_path();
    if !home_dir.exists() {
        std::fs::create_dir_all(home_dir.clone()).unwrap();
    }

    let deps = load_local_modules(&home_dir);
    let data = fs::read(path).expect("read file error");
    let program = str::from_utf8(&data).unwrap();

    let compiled_program =
        compile_inner_program_with_deps(sender.address(), &program, deps.clone());
    // save modules
    compiled_program.modules.into_iter().for_each(|module| {
        let mut data: Vec<u8> = vec![];
        module.serialize(&mut data).unwrap();
        let path = home_dir.join(format!("{}", module.self_id()));
        // println!("path {:?}", path);
        let mut file = File::create(path).expect("should create file");
        file.write_all(&data).expect("should write file");
    });

    let program = compile_program_with_deps(sender.address(), &program, args, deps);
    // create signed transaction
    sender.create_signed_txn_with_program(
        program,
        seq_num,
        gas_costs::TXN_RESERVED, // this is a default for gas
        0,                       // this is a default for gas
    )
}

fn generate_sub_command_get_access_path<'a, 'b>() -> App<'a, 'b> {
    let subcommand = SubCommand::with_name("get_access_path")
        .about("get status")
        .arg(
            Arg::with_name("address")
                .short("a")
                .long("address")
                .takes_value(true)
                .help("account address."),
        )
        .arg(
            Arg::with_name("pubkey")
                .short("p")
                .long("pubkey")
                .takes_value(true)
                .help("account public key."),
        );
    subcommand
}

fn deal_command_get_access_path(args: &ArgMatches) {
    //    let address = if let Some(address) = args.value_of("address") {
    //        Some(AccountAddress::from_hex_literal(address).unwrap())
    //    } else if let Some(pubkey) = args.value_of("pubkey") {
    //        let pubkey =
    //            Ed25519PublicKey::try_from(hex::decode(&pubkey[2..]).unwrap().as_slice()).unwrap();
    //        Some(AccountAddress::from_public_key(&pubkey))
    //    } else {
    //        None
    //    };
    //    let address = address.expect("should provide account address or public key.");
    //    let access_path = AccessPath::new_for_account(address);
    //    let access_path: Vec<u8> = SimpleSerializer::serialize(&access_path).unwrap();
    //    let access_path = hex::encode(access_path);
    //    print!("0x{}", access_path);
}

fn generate_sub_command_decode<'a, 'b>() -> App<'a, 'b> {
    let subcommand = SubCommand::with_name("decode")
        .about("decode account resource")
        .arg(
            Arg::with_name("data")
                .short("d")
                .long("data")
                .takes_value(true)
                .help("account resource hex."),
        );
    subcommand
}

fn deal_command_decode(args: &ArgMatches) {
    //    #[derive(Deserialize, Serialize, Debug)]
    //    struct Status {
    //        balance: String,
    //        sequence_number: u64,
    //    }
    //
    //    if let Some(data) = args.value_of("data") {
    //        let data = hex::decode(&data[2..]).unwrap();
    //        let account_resource: AccountResource = SimpleDeserializer::deserialize(&data).unwrap();
    //        let balance = account_resource.balance();
    //        let balance = balance.to_be_bytes();
    //        let balance = hex::encode(balance);
    //
    //        let account_status = Status {
    //            balance,
    //            sequence_number: account_resource.sequence_number(),
    //        };
    //        print!("{}", serde_json::to_string(&account_status).unwrap());
    //    }
}

fn generate_sub_command_get_address<'a, 'b>() -> App<'a, 'b> {
    let subcommand = SubCommand::with_name("get_address")
        .about("get address by public key")
        .arg(
            Arg::with_name("pubkey")
                .short("p")
                .long("pubkey")
                .takes_value(true)
                .help("account public key."),
        );
    subcommand
}

fn deal_command_get_address(args: &ArgMatches) {
    //    let pubkey = args.value_of("pubkey").expect("should provide public key");
    //    let pubkey = Ed25519PublicKey::try_from(hex::decode(&pubkey[2..]).unwrap().as_slice()).unwrap();
    //    let address = AccountAddress::from_public_key(&pubkey);
    //    let address = address.to_vec();
    //    let address = hex::encode(address);
    //    print!("0x{}", address);
}

fn generate_sub_command_get_public_key<'a, 'b>() -> App<'a, 'b> {
    let subcommand = SubCommand::with_name("get_public_key")
        .about("get public key by private key")
        .arg(
            Arg::with_name("key")
                .short("k")
                .long("key")
                .takes_value(true)
                .help("account private key."),
        );
    subcommand
}

fn deal_command_get_public_key(args: &ArgMatches) {
    //    let private_key = args.value_of("key").expect("should provide private key");
    //    let private_key = hex::decode(&private_key[2..]).expect("private key is invalid");
    //    let private_key =
    //        Ed25519PrivateKey::try_from(private_key.as_slice()).expect("private key is invalid");
    //    let key_pair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey> = KeyPair::from(private_key);
    //    let public_key = key_pair.public_key.to_bytes();
    //    let public_key = hex::encode(public_key);
    //    print!("0x{}", public_key);
}

fn generate_sub_command_account<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("account")
        .about("about account operator")
        .arg(
            Arg::with_name("create")
                .short("c")
                .long("create")
                .takes_value(false)
                .help("create account with rand"),
        )
        .arg(
            Arg::with_name("recover")
                .short("r")
                .long("recover")
                .takes_value(true)
                .help("recover account by private key"),
        )
}

fn deal_command_account(args: &ArgMatches) {
    if args.is_present("create") {
        println!("{}", Account::new());
    } else if args.is_present("recover") {
        let account = args
            .value_of("recover")
            .map(|input| hex::decode(&input[2..]))
            .map(|data| PrivateKey::from_slice(&data.unwrap()))
            .map(|key| KeyPair::new(key.unwrap()))
            .map(|pair| Account::from_keypair(pair))
            .expect("unknown private key");
        println!("{}", account);
    } else {
        println!("{}", Account::new());
    }
}
