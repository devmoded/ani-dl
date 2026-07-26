mod cli;
mod engine;

use console::style;
use inquire::InquireError;

pub async fn run() {
    match engine::run_engine().await {
        Ok(()) => {  }
        Err(err) => {
            if let Some(inquire_err) = err.downcast_ref::<InquireError>() {
                match inquire_err {
                    InquireError::OperationCanceled => {}
                    _ => {}
                }
            } else {
                eprintln!("{}", style(err.to_string()).red());
            }
        }
    }
}
