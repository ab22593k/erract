use erract::prelude::*;
use erract::{has_permanent, has_retryable, Error, ErrorKind, ErrorStatus};

// Helper functions for creating common error types
fn not_found_error(message: impl Into<String>) -> Error {
    Error::permanent(ErrorKind::NotFound, message)
}

fn temporary_error(message: impl Into<String>) -> Error {
    Error::temporary(ErrorKind::Timeout, message)
}

fn unexpected_error(message: impl Into<String>) -> Error {
    Error::temporary(ErrorKind::Unexpected, message)
}

fn simulate_database_lookup(user_id: u32) -> erract::Result<Option<String>> {
    if user_id == 0 {
        Err(not_found_error(format!("user not found: {user_id}")).raise())
    } else if user_id == 999 {
        Err(temporary_error("database connection timeout").raise())
    } else {
        Ok(Some(format!("User {user_id}")))
    }
}

fn fetch_user_data(user: &str) -> erract::Result<String> {
    Err(
        unexpected_error(format!("failed to fetch data for user: {user}"))
            .with_context("user", user)
            .with_context("operation", "fetch_user_data")
            .raise(),
    )
}

fn process_user(user_id: u32) -> erract::Result<String> {
    let user =
        simulate_database_lookup(user_id).or_raise(|| unexpected_error("failed to lookup user"))?;

    let user = user.ok_or_else(|| not_found_error(format!("user {user_id} not found")).raise())?;

    let data = fetch_user_data(&user).or_raise(|| unexpected_error("failed to fetch user data"))?;

    Ok(format!("Processed: {user} - Data: {data}"))
}

fn demonstrate_error_tree() {
    println!("=== Demonstrating Error Tree ===");

    fn inner() -> erract::Result<()> {
        bail!(Error::permanent(ErrorKind::NotFound, "inner error"));
    }

    fn middle() -> erract::Result<()> {
        inner().or_raise(|| Error::temporary(ErrorKind::Unexpected, "middle wrapper"))?;
        Ok(())
    }

    fn outer() -> erract::Result<()> {
        middle().or_raise(|| Error::permanent(ErrorKind::Unexpected, "outer wrapper"))?;
        Ok(())
    }

    if let Err(exn) = outer() {
        println!("Error tree:");
        println!("{exn:?}");
        println!("\nFrame count: {}", count_frames(&exn));
        println!("Has retryable: {}", has_retryable(&exn));
        println!("Has permanent: {}", has_permanent(&exn));
    }
}

fn demonstrate_context() {
    println!("\n=== Demonstrating Context ===");

    let error = Error::permanent(ErrorKind::NotFound, "resource not found")
        .with_context("resource_id", "user_123")
        .with_context("resource_type", "user")
        .with_operation("fetch_user");

    println!("Error with context:");
    println!("{error}");
    println!("\nIs retryable: {}", error.is_retryable());
    println!("Kind: {}", error.kind());
    println!("Status: {}", error.status());
}

fn demonstrate_retry_logic() {
    println!("\n=== Demonstrating Retry Logic ===");

    let retryable_error = Error::temporary(ErrorKind::Timeout, "operation timed out");
    let permanent_error = Error::permanent(ErrorKind::NotFound, "resource not found");

    println!(
        "Retryable error is_retryable(): {}",
        retryable_error.is_retryable()
    );
    println!(
        "Permanent error is_retryable(): {}",
        permanent_error.is_retryable()
    );
}

fn demonstrate_builder() {
    println!("\n=== Demonstrating Error Builder ===");

    let error = Error::builder(
        ErrorKind::Validation,
        ErrorStatus::Permanent,
        "validation failed",
    )
    .with_operation("validate_input")
    .with_context("field", "email")
    .with_context("reason", "invalid format")
    .build();

    println!("Built error:");
    println!("{error}");
}

fn demonstrate_machine_readable() {
    println!("\n=== Demonstrating Machine-Readable Formatting ===");

    let error = Error::permanent(ErrorKind::NotFound, "user not found")
        .with_context("user_id", "123")
        .with_context("operation", "fetch_user")
        .with_operation("user_service");

    println!("Human-readable:");
    println!("{error}");
    println!("\nMachine-readable:");
    println!("{}", error.to_machine_string());
    println!("\nJSON format:");
    println!("{}", error.to_json());
}

fn demonstrate_from_conversions() {
    println!("\n=== Demonstrating From Conversions ===");

    // Simulate an IO error
    use std::io::{Error as IoError, ErrorKind as IoErrorKind};

    let io_error = IoError::new(IoErrorKind::NotFound, "file.txt not found");
    let erract_error: Error = io_error.into();

    println!("IO error converted:");
    println!("  Kind: {}", erract_error.kind());
    println!("  Status: {}", erract_error.status());
    println!("  Message: {}", erract_error.message());

    // Parse int error
    let parse_error: Result<u32, _> = "abc".parse();
    match parse_error {
        Ok(_) => {}
        Err(e) => {
            let erract_error: Error = e.into();
            println!("\nParse error converted:");
            println!("  Kind: {}", erract_error.kind());
            println!("  Status: {}", erract_error.status());
        }
    }
}

fn main() {
    println!("erract - Structured Error Handling Demo\n");

    demonstrate_context();
    demonstrate_retry_logic();
    demonstrate_builder();
    demonstrate_machine_readable();
    demonstrate_from_conversions();
    demonstrate_error_tree();

    println!("\n=== Processing Users ===");

    for user_id in [1, 0, 42, 999] {
        print!("User {user_id}: ");
        match process_user(user_id) {
            Ok(result) => println!("{result}"),
            Err(exn) => {
                println!("Error occurred:");
                println!("{exn:?}");
            }
        }
    }
}
