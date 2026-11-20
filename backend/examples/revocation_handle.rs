use ocl_api::credential::{is_credential_id, parse_issuer, revocation_handle};
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("Usage: revocation_handle <lowercase-issuer-address> <credential-id>");
        std::process::exit(2);
    }
    let Some(issuer) = parse_issuer(&args[0]) else {
        eprintln!("Invalid issuer");
        std::process::exit(2);
    };
    if !is_credential_id(&args[1]) {
        eprintln!("Invalid credential identifier");
        std::process::exit(2);
    }
    println!("{:#x}", revocation_handle(issuer, &args[1]));
}
