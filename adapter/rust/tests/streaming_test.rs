// adapter/rust/tests/streaming_test.rs
// TODO: Re-enable when streaming module is implemented
// use wireframe_adapter::streaming::parse_sse_line;

#[test]
fn test_parse_sse_text_delta() {
    // let line = "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n";
    // let result = parse_sse_line(line, "openai");
    // assert!(result.is_some());
}

#[test]
fn test_parse_sse_done() {
    // let line = "data: [DONE]\n\n";
    // let result = parse_sse_line(line, "openai");
    // assert!(result.is_some());
}
