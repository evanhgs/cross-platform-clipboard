use std::backtrace::Backtrace;

pub fn report_error(context: &str, error: &anyhow::Error) {
    tracing::error!(
        context,
        error = %error,
        error_chain = %format!("{error:#}"),
        backtrace = %Backtrace::force_capture(),
        "operation failed"
    );
}
