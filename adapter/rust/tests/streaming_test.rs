// adapter/rust/tests/streaming_test.rs
use wireframe_adapter::streaming::parse_sse_line;

#[test]
fn test_parse_sse_text_delta() {
    let line = "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n";
    let result = parse_sse_line(line, "openai");
    assert!(result.is_some());
}

#[test]
fn test_parse_sse_done() {
    let line = "data: [DONE]\n\n";
    let result = parse_sse_line(line, "openai");
    assert!(result.is_some());
}
