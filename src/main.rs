use error_stack::Report;
use std::fmt::{Debug, Display};
use std::sync::Arc;
use thalia::base::error::MainError;
use thalia::config::get_config;
use thalia::startup::Application;
use thalia::telemetry::{get_tracing_subscriber, init_tracing_subscriber};
use tokio::task::JoinError;

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(error.cause_chain = ?e, error.message= %e, "{} task failed to complete", task_name)
        }
        Err(e) => {
            tracing::error!(error.cause_chain = ?e, error.message= %e, "{} failed", task_name)
        }
    }
}

#[actix_web::main]
async fn main() -> Result<(), Report<MainError>> {
    let config = get_config().map_err(|e| {
        Report::new(MainError::Runtime {
            value: format!("{}", e),
        })
    })?;
    let config = Arc::new(config);

    let subscriber = get_tracing_subscriber(
        config.bunyan_formatting_name.clone(),
        config.env_filter.clone(),
        std::io::stdout,
    );
    init_tracing_subscriber(subscriber);

    let app = Application::build(&config.clone()).await.map_err(|e| {
        Report::new(MainError::Runtime {
            value: format!("{}", e),
        })
    })?;
    let app_task = tokio::spawn(app.run_until_stopped());

    tokio::select! {
        o = app_task => {report_exit("api", o);}
    }

    Ok(())
}
