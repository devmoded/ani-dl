mod cli;
mod engine;

use console::style;
use inquire::InquireError;

pub async fn run() {
    match engine::run_engine().await {
        // TODO: Улучшить обработку ошибок
        Ok(()) => {  }
        Err(err) => {
            if let Some(inquire_err) = err.downcast_ref::<InquireError>() {
                match inquire_err {
                    InquireError::OperationCanceled => {}
                    _ => {}
                }
            } else {
                eprintln!("Ошибка: {}:\n\n{}", style(err.to_string()).red(), err.root_cause().to_string());
            }
        }
    }
}
