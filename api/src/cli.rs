//! `ludusd` alone runs the daemon; with arguments it does one admin task and exits.

use crate::service::Service;

const USAGE: &str = "\
usage: ludusd                           run the daemon
       ludusd user list
       ludusd user add <email> [name]    create an account and print its password
       ludusd user password <user>       issue a new password, end every session
       ludusd user activate <user>       let a pending account in

<user> is an email, or the id when several accounts share that email.";

pub async fn run(args: &[String], service: Service) -> anyhow::Result<()> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["user", "list"] => {
            for (user, logins) in service.admin_users().await? {
                println!(
                    "{}  {:<7}  {:<30}  {:<20}  {}",
                    user.id,
                    user.status.as_str(),
                    user.email,
                    user.display_name,
                    logins.join(", ")
                );
            }
        }
        ["user", "add", email, name @ ..] => {
            let name = (!name.is_empty()).then(|| name.join(" "));
            let (user, password) = service.admin_add(email, name).await?;
            println!("created {} ({})", user.email, user.id);
            print_password(&password);
        }
        ["user", "password", key] => {
            let (user, password) = service.admin_reset_password(key).await?;
            println!(
                "new password for {} - every session it had has ended",
                user.email
            );
            print_password(&password);
        }
        ["user", "activate", key] => {
            let user = service.admin_activate(key).await?;
            println!("{} is active", user.email);
        }
        _ => {
            eprintln!("{USAGE}");
            std::process::exit(2);
        }
    }
    Ok(())
}

/// Shown once and stored nowhere but as a hash, so this is the only chance to copy it.
fn print_password(password: &str) {
    println!("\n    {password}\n\nIt is not stored anywhere readable; copy it now.");
}
