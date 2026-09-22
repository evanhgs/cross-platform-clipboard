pub fn report_error(context: &str, error: &anyhow::Error) {
    tracing::error!(
        context,
        error = %error,
        "operation failed"
    );
}
