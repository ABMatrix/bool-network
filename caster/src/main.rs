use canonical_serialization::{SimpleDeserializer, SimpleSerializer};
use clap::{App, Arg, ArgMatches, SubCommand};
use core::convert::TryFrom;
use crypto::{signing, PrivateKey, PublicKey, signing::KeyPair};
use hex;
use serde::{Deserialize, Serialize};
use std::fs;
use vm::types::{
    AccessPath, AccountAddress, account_config::AccountResource,
};
use mock::common;
use mock::account::Account;

fn main() {
    let args = App::new("Libra Cli")
        .version("0.2.1")
        .author("Jason")
        .about("Libra cli for making transaction code or decoding account status.")
        .subcommand(generate_sub_command_tx())
        .subcommand(generate_sub_command_get_access_path())
        .subcommand(generate_sub_command_decode())
        .subcommand(generate_sub_command_get_address())
        .subcommand(generate_sub_command_get_public_key())
        .get_matches();

    if let (command, Some(matches)) = args.subcommand() {
        match command {
            "tx" => deal_command_make_tx(matches),
            "get_access_path" => deal_command_get_access_path(matches),
            "decode" => deal_command_decode(matches),
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
            Arg::with_name("complied_file")
                .short("cf")
                .long("complied_file")
                .takes_value(true)
                .help("complied file path."),
        );
    subcommand
}

fn deal_command_make_tx(args: &ArgMatches) {
    let key_pair = args
        .value_of("key")
        .map(|input| hex::decode(&input[2..]))
        .map(|data| PrivateKey::from_slice(&data.unwrap()))
        .map(|key| KeyPair::new(key.unwrap()))
        .expect("should private key");

    let receiver = args
        .value_of("recipient")
        .map(|input| hex::decode(&input[2..]))
        .map(|data| {
            println!("data {:?}", data);
            PublicKey::from_slice(&data.unwrap())
        })
        .expect("should has public key ").unwrap();

    let num_coins = args
        .value_of("value")
        .expect("should provide number of coins")
        .parse()
        .unwrap();

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
                common::create_account_txn(
                    &Account::from_keypair(key_pair),
                    &Account::mock(&receiver.to_slice()),
                sequence_number,
                num_coins)
            },
            "mint" => {
                common::mint_txn(
                    &Account::from_keypair(key_pair),
                    &Account::mock(&receiver.to_slice()),
                sequence_number,
                num_coins
                )
            }
            "transfer" => {
                common::peer_to_peer_txn(
                    &Account::from_keypair(key_pair),
                    &Account::mock(&receiver.to_slice()),
                sequence_number,
                num_coins
                )
            },
            "publish_module" => {
                let path = args.value_of("complied_file").unwrap();
                let data = fs::read(path).expect("read file error");
                serde_json::from_slice(&data).expect("decode program error.")
            }
            _ => unimplemented!(),
        };

    let se_txn = SimpleSerializer::<Vec<u8>>::serialize(&signed_txn).unwrap();
    let hex = hex::encode(se_txn);
    print!("0x{}", hex);
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
