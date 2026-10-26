#![forbid(unsafe_code)]

use rev::buffer::Buffers;

// Empty and loaded stores must retain exact UTF-8 text and separator counts.
#[test]
fn initial_store() {
    let mut buffers = Buffers::new().unwrap();
    let initial = buffers.initial();
    let buffer = buffers.get(initial).unwrap();
    assert_eq!(buffer.id(), initial);
    assert_eq!(
        (buffer.len_chars(), buffer.len_bytes(), buffer.len_lines()),
        (0, 0, 1)
    );
    assert_eq!(buffer.slice(&buffer.span(0..0).unwrap()).unwrap(), "");

    let cases = [
        ("empty", "", 0, 0, 1),
        ("unicode", "a🙂é", 3, 7, 1),
        ("multiline", "a\nβ\r\n", 5, 6, 3),
        ("final text", "first\nlast", 10, 10, 2),
    ];
    for (name, text, chars, bytes, lines) in cases {
        let id = buffers.create(text).unwrap();
        let buffer = buffers.get(id).unwrap();
        assert_eq!(
            (buffer.len_chars(), buffer.len_bytes(), buffer.len_lines()),
            (chars, bytes, lines),
            "{name}"
        );
        let span = buffer.span(0..chars).unwrap();
        assert_eq!(buffer.slice(&span).unwrap(), text, "{name}");
    }
    assert_eq!(buffers.get(initial).unwrap().len_chars(), 0);
}
