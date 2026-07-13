fn exact_multiline_latex_response() -> &'static str {
    concat!(
        "\\[\n\\boxed{\ne^{i\\pi}+1=0\n}\n\\]\n\n",
        "\\[\n\\int_{-\\infty}^{\\infty} e^{-x^2}\\,dx=\\sqrt{\\pi}\n\\]\n\n",
        "\\[\nx=\\frac{-b\\pm\\sqrt{b^2-4ac}}{2a}\n\\]\n\n",
        "\\[\n\\nabla\\cdot\\mathbf{E}=\\frac{\\rho}{\\varepsilon_0}\n\\]\n\n",
        "\\[\n\\frac{\\partial \\psi}{\\partial t}\n=\n",
        "\\alpha\\frac{\\partial^2\\psi}{\\partial x^2}\n\\]",
    )
}

#[test]
fn exact_multiline_response_renders_all_five_equations() {
    let mut renderer = IncrementalMarkdownRenderer::new(Some(90));
    let rendered = lines_to_string(&renderer.update(exact_multiline_latex_response()));

    assert_eq!(rendered.matches("┌─ math").count(), 5, "{rendered}");
    assert!(!rendered.contains("$$"), "{rendered}");
    assert!(!rendered.contains(r"\partial"), "{rendered}");
    assert!(rendered.contains('∂'), "{rendered}");
    assert!(rendered.contains('α'), "{rendered}");
}

#[test]
fn every_streaming_prefix_converges_to_the_full_math_render() {
    let response = exact_multiline_latex_response();
    let mut renderer = IncrementalMarkdownRenderer::new(Some(90));

    for end in response
        .char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(response.len()))
    {
        let _ = renderer.update(&response[..end]);
    }

    let incremental = renderer.update(response);
    let full = with_streaming_render_context(|| render_markdown_with_width(response, Some(90)));
    assert_eq!(incremental, full);
    assert_eq!(lines_to_string(&incremental).matches("┌─ math").count(), 5);
}

#[test]
fn streaming_math_never_invokes_the_synchronous_image_toolchain() {
    latex_image::reset_test_render_attempts();
    let mut renderer = IncrementalMarkdownRenderer::new(Some(90));
    let _ = renderer.update(exact_multiline_latex_response());
    assert_eq!(latex_image::test_render_attempts(), 0);

    latex_image::reset_test_render_attempts();
    let _ = render_markdown(r"$$x^2$$");
    assert!(
        latex_image::test_render_attempts() > 0,
        "completed non-streaming Image mode should attempt the configured image renderer"
    );
}
